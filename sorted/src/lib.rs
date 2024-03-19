use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    let item = parse_macro_input!(input as syn::Item);
    if matches!(item, syn::Item::Enum(_)) {
        item.to_token_stream().into()
    } else {
        syn::Error::new(Span::call_site(), "expected enum or match expression").to_compile_error().into()
    }
}
