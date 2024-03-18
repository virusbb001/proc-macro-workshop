use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree, Literal, Span, Delimiter};
use quote::{quote, ToTokens};
use syn::{Ident, parse::Parse, parse_macro_input, Token, LitInt, braced, parenthesized};

#[derive(Debug)]
struct Seq {
    ident_replace: Ident,
    start: LitInt,
    end: LitInt,
    tokens: TokenStream2
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident_replace = input.parse::<Ident>()?;
        input.parse::<Token![in]>()?;
        let start = input.parse::<LitInt>()?;
        input.parse::<Token![..]>()?;
        let end = input.parse::<LitInt>()?;
        let content;
        braced!(content in input);
        let tokens = content.parse::<TokenStream2>()?;

        Ok(Seq {
            ident_replace,
            start,
            end,
            tokens,
        })
    }
}

#[derive(Debug, Clone)]
enum SeqTokenTree {
    Raw(TokenTree),
    Ident(Vec<syn::Ident>),
    Group(SeqGroup),
    Trees(SeqTrees),
}

impl SeqTokenTree {
    fn to_token_tree(
        &self,
        start: usize,
        end: usize,
        ident_replace: &Ident,
    ) -> TokenStream2 {
        match self {
            SeqTokenTree::Raw(tree) => {
                if let TokenTree::Ident(ident) = tree {
                    if ident == ident_replace {
                        TokenTree::Literal(Literal::usize_unsuffixed(start)).into()
                    } else {
                        TokenTree::Ident(ident.clone()).into()
                    }
                } else {
                    tree.clone().into()
                }
            },
            SeqTokenTree::Ident(idents) => {
                let new_ident_name = idents.iter().map(|ident| {
                    if ident == ident_replace {
                        start.to_string()
                    } else {
                        ident.to_string()
                    }
                }).collect::<Vec<_>>().join("");
                TokenTree::Ident(proc_macro2::Ident::new(&new_ident_name, Span::call_site())).into()
            },
            SeqTokenTree::Group(seq_group) => {
                let stream = seq_group.trees.token_trees.iter().map(|v| v.to_token_tree(start, end, ident_replace)).collect::<TokenStream2>();
                let mut group = proc_macro2::Group::new(seq_group.delimiter, stream);
                group.set_span(seq_group.span);
                TokenTree::Group(group).into()
            },
            SeqTokenTree::Trees(trees) => {
                if trees.to_expand {
                    (start..end).map(|n| {
                        trees.token_trees.iter().map(|v| v.to_token_tree(n, n+1, ident_replace)).collect::<TokenStream2>()
                    }).collect::<TokenStream2>()
                } else {
                    trees.token_trees.iter().map(|v| v.to_token_tree(start, end, ident_replace)).collect::<TokenStream2>()
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
struct SeqTrees {
    pub to_expand: bool,
    pub token_trees: Vec<SeqTokenTree>,
}

#[derive(Debug, Clone)]
struct SeqGroup {
    delimiter: Delimiter,
    trees: SeqTrees,
    span: Span,
}

impl SeqTrees {
    fn has_to_expand(&self) -> bool {
        if self.to_expand {
            true
        } else {
            self.token_trees.iter().any(|tree| {
                match tree {
                    SeqTokenTree::Raw(_) => false,
                    SeqTokenTree::Ident(_) => false,
                    SeqTokenTree::Group(group) => group.trees.has_to_expand(),
                    SeqTokenTree::Trees(tree) => tree.has_to_expand(),
                }
            })
        }
    }
}

impl Parse for SeqTrees {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut trees = Vec::<SeqTokenTree>::new();
        loop {
            if input.is_empty() {
                break;
            }
            if input.peek(Token![#]) && input.peek2(syn::token::Paren) {
                let content;
                input.parse::<Token![#]>()?;
                parenthesized!(content in input);
                input.parse::<Token![*]>()?;
                let mut seq_trees = content.parse::<SeqTrees>()?;
                seq_trees.to_expand = true;
                trees.push(SeqTokenTree::Trees(seq_trees));
                continue;
            }
            let tree = input.parse::<TokenTree>()?;
            if input.peek(Token![~]) {
                if let TokenTree::Ident(ident) = tree {
                    input.parse::<Token![~]>()?;
                    let next = input.parse::<Ident>()?;
                    trees.push(SeqTokenTree::Ident(vec![ident, next]))
                } else {
                    trees.push(SeqTokenTree::Raw(tree));
                }
            } else if let TokenTree::Group(group) = tree {
                let group_trees = syn::parse2::<SeqTrees>(group.stream())?;
                let seq_group = SeqGroup {
                    delimiter: group.delimiter(),
                    trees: group_trees,
                    span: group.span(),
                };
                trees.push(SeqTokenTree::Group(seq_group));
            } else {
                trees.push(SeqTokenTree::Raw(tree));
            }
        }
        Ok(Self {
            to_expand: false,
            token_trees: trees
        })
    }
}

struct SeqGroupToReplace<'a, 'b> {
    start: usize,
    end: usize,
    group: &'a SeqTrees,
    ident: &'b Ident,
}

impl<'a, 'b> SeqGroupToReplace<'a, 'b> {
    pub fn new(
        start: usize,
        end: usize,
        group: &'a SeqTrees,
        ident: &'b Ident,
    ) -> Self {
        SeqGroupToReplace {
            start,
            end,
            group,
            ident
        }
    }
}

impl ToTokens for SeqGroupToReplace<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let tokentree_iter = SeqTokenTree::Trees(self.group.clone()).to_token_tree(self.start, self.end, self.ident);

        tokens.extend(tokentree_iter);
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let Seq {
        ident_replace,
        start,
        end,
        tokens,
    } = parse_macro_input!(input as Seq);
    let start = start.base10_parse::<usize>();
    let end = end.base10_parse::<usize>();
    let (Ok(start), Ok(end)) = (start.clone(), end.clone()) else {
        return [start, end]
            .into_iter()
            .filter_map(|v| v.err())
            .map(|v| v.into_compile_error())
            .collect::<TokenStream2>()
            .into();
    };

   let mut seq_token_tree = match syn::parse2::<SeqTrees>(tokens.clone()) {
        Ok(data) => data,
        Err(err) => return err.into_compile_error().into(),
    };
    seq_token_tree.to_expand = seq_token_tree.to_expand || !seq_token_tree.has_to_expand();
    
    let tokens = SeqGroupToReplace::new(start, end, &seq_token_tree, &ident_replace).to_token_stream();

    quote! {
        #tokens
    }.into()
}
