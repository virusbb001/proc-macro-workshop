use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree, Literal, Group};
use quote::quote;
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

    let tokens = (start..end).map(|n| {
        tokens.clone().into_iter().map(|tokentree| replace_tokens(tokentree, &ident_replace, n)).collect::<TokenStream2>()
    }).collect::<TokenStream2>();

    quote! {
        #tokens
    }.into()
}

fn replace_tokens(
    token_tree: TokenTree,
    ident_replace: &Ident,
    n: usize,
) -> TokenTree {
    match token_tree {
        TokenTree::Group(group) => {
            let delimiter = group.delimiter();
            let stream = group.stream().into_iter().map(|token_tree| replace_tokens(token_tree, ident_replace, n)).collect::<TokenStream2>();
            let span = group.span();
            let mut group = Group::new(delimiter, stream);
            group.set_span(span);
            TokenTree::Group(group)
        },
        TokenTree::Ident(ident) => {
            if ident == *ident_replace {
                TokenTree::Literal(Literal::usize_unsuffixed(n))
            } else {
                TokenTree::Ident(ident)
            }
        },
        TokenTree::Punct(_) => token_tree,
        TokenTree::Literal(_) => token_tree,
    }
}
