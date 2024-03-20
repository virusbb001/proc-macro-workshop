use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::parse_macro_input;

#[derive(Debug)]
struct WrongLocations {
    target: String,
    expected: String,
    span: Span,
}

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    let item = parse_macro_input!(input as syn::Item);
    let syn::Item::Enum(item_enum) = item else {
        return syn::Error::new(Span::call_site(), "expected enum or match expression")
            .to_compile_error()
            .into()
    };
    let mut sorted_variants = item_enum
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .enumerate()
        .collect::<Vec<_>>();
    sorted_variants.sort_by_key(|v| v.1);
    let wrong_positions = sorted_variants
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            sorted_variants
                .iter()
                .skip(i + 1)
                .find(|v| v.0 < item.0)
                .map(|other| WrongLocations {
                    target: item.1.to_string(),
                    expected: other.1.to_string(),
                    span: item.1.span(),
                })
        })
        .map(|wrong| {
            syn::Error::new(
                wrong.span,
                format!("{} should sort before {}", wrong.target, wrong.expected),
            )
            .to_compile_error()
        })
        .collect::<TokenStream2>();
    if wrong_positions.is_empty() {
        item_enum.to_token_stream().into()
    } else {
        wrong_positions.into()
    }
}
