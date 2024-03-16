use std::iter::Peekable;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree, Literal, Group, Span};
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

struct ReplaceTokenState<'a, 'b, T>
    where T: Iterator<Item=TokenTree> {
        iter: &'a mut Peekable<T>,
        ident_replace: &'b Ident,
        n: usize,
}

trait ReplaceToken<T: std::iter::Iterator<Item = proc_macro2::TokenTree>> {
    fn replace_token<'a>(
        &mut self,
        ident_replace: &'a Ident,
        n: usize
    ) -> ReplaceTokenState<'_, 'a, T>;
}

impl<T> ReplaceToken<T> for Peekable<T>
    where T: Iterator<Item=TokenTree>
{
    fn replace_token<'a>(&mut self, ident_replace: &'a Ident, n: usize) -> ReplaceTokenState<'_, 'a, T> {
        ReplaceTokenState {
            iter: self,
            ident_replace,
            n
        }
    }
}

impl<T: std::iter::Iterator<Item = proc_macro2::TokenTree>> Iterator for ReplaceTokenState<'_, '_, T> {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        let token_tree = self.iter.next()?;
        let token_tree = match token_tree {
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let stream = group.stream().into_iter().peekable().replace_token(self.ident_replace, self.n).collect::<TokenStream2>();
                let span = group.span();
                let mut group = Group::new(delimiter, stream);
                group.set_span(span);
                TokenTree::Group(group)
            },
            TokenTree::Ident(ident) => {
                if let Some(TokenTree::Punct(next)) = self.iter.peek() {
                    if next.as_char() == '~' {
                        self.iter.next()?; // consume ~
                        let TokenTree::Ident(next) = self.iter.next().unwrap() else {
                            panic!("element followed after ~ is not ident");
                        };
                        let next = if next == *self.ident_replace {
                            self.n.to_string()
                        } else {
                            next.to_string()
                        };
                        TokenTree::Ident(Ident::new(&format!("{}{}", ident, next), Span::call_site()))
                    } else if ident == *self.ident_replace {
                        TokenTree::Literal(Literal::usize_unsuffixed(self.n))
                    } else {
                        TokenTree::Ident(ident)
                    }
                } else if ident == *self.ident_replace {
                    TokenTree::Literal(Literal::usize_unsuffixed(self.n))
                } else {
                    TokenTree::Ident(ident)
                }
            },
            TokenTree::Punct(_) => token_tree,
            TokenTree::Literal(_) => token_tree,
        };
        Some(token_tree)
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
        tokens.clone().into_iter().peekable().replace_token(&ident_replace, n).collect::<TokenStream2>()
    }).collect::<TokenStream2>();

    quote! {
        #tokens
    }.into()
}
