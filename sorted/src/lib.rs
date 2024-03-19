use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    let item = parse_macro_input!(input as syn::Item);
    item.to_token_stream().into()
}
