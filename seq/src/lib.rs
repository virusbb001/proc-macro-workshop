use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree, Literal, Group, Span, Delimiter};
use quote::{quote, ToTokens};
use syn::{Ident, parse::Parse, parse_macro_input, Token, LitInt, braced};

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
}

impl SeqTokenTree {
    fn to_token_tree(
        &self,
        n: usize,
        ident_replace: &Ident,
    ) -> TokenTree {
        match self {
            SeqTokenTree::Raw(tree) => {
                if let TokenTree::Ident(ident) = tree {
                    if ident == ident_replace {
                        TokenTree::Literal(Literal::usize_unsuffixed(n))
                    } else {
                        TokenTree::Ident(ident.clone())
                    }
                } else {
                    tree.clone()
                }
            },
            SeqTokenTree::Ident(idents) => {
                let new_ident_name = idents.iter().map(|ident| {
                    if ident == ident_replace {
                        n.to_string()
                    } else {
                        ident.to_string()
                    }
                }).collect::<Vec<_>>().join("");
                TokenTree::Ident(proc_macro2::Ident::new(&new_ident_name, Span::call_site()))
            },
            SeqTokenTree::Group(seq_group) => {
                let stream = seq_group.trees.token_trees.iter().map(|v| v.to_token_tree(n, ident_replace)).collect::<TokenStream2>();
                let mut group = proc_macro2::Group::new(seq_group.delimiter, stream);
                group.set_span(seq_group.span);
                TokenTree::Group(group)
            },
        }
    }
}

#[derive(Debug, Clone)]
struct SeqTrees {
    pub token_trees: Vec<SeqTokenTree>,
}

#[derive(Debug, Clone)]
struct SeqGroup {
    delimiter: Delimiter,
    trees: SeqTrees,
    span: Span,
}

impl Parse for SeqTrees {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut trees = Vec::<SeqTokenTree>::new();
        loop {
            if input.is_empty() {
                break;
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
            token_trees: trees
        })
    }
}

struct SeqGroupToReplace<'a, 'b> {
    n: usize,
    group: &'a SeqTrees,
    ident: &'b Ident,
}

impl<'a, 'b> SeqGroupToReplace<'a, 'b> {
    pub fn new(
        n: usize,
        group: &'a SeqTrees,
        ident: &'b Ident,
    ) -> Self {
        SeqGroupToReplace {
            n,
            group,
            ident
        }
    }
}

impl ToTokens for SeqGroupToReplace<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let tokentree_iter = self.group.token_trees.clone().into_iter().map(|tree| tree.to_token_tree(self.n, self.ident));

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

   let seq_token_tree = match syn::parse2::<SeqTrees>(tokens.clone()) {
        Ok(data) => data,
        Err(err) => return err.into_compile_error().into(),
    };
    
    let tokens = (start..end).map(|n| {
        SeqGroupToReplace::new(n, &seq_token_tree, &ident_replace).to_token_stream()
    }).collect::<TokenStream2>();

    quote! {
        #tokens
    }.into()
}
