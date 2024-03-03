use proc_macro::TokenStream;

#[proc_macro_derive(CustomDebug)]
pub fn derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
