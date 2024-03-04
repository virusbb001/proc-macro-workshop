use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};
use syn::spanned::Spanned;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let ident_litstr = ident.to_string();
    let Data::Struct(input_struct) = input.data else {
        return syn::Error::new(input.span(), "CustomDebug is not used for struct").into_compile_error().into();
    };

    let field_call = input_struct.fields.iter().filter_map(|field| {
        let ident = field.ident.as_ref()?;
        let ident_str = ident.to_string();
        Some(quote! {
            .field(#ident_str, &self.#ident)
        })
    });
    quote! {
        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                f.debug_struct(#ident_litstr)
                    #(#field_call)*
                    .finish()
            }
        }
    }.into()
}
