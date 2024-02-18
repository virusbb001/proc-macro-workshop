use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{self, parse_macro_input, DeriveInput, Data, Fields};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;
    let Data::Struct(data) = ast.data else {
        panic!("Builder is not used for struct");
    };
    let Fields::Named(_fields) = data.fields else {
        panic!("Builder is not used for struct with named fields");
    };
    let builder_name = syn::Ident::new(&format!("{}Builder", ident), Span::call_site());
    proc_macro::TokenStream::from(quote! {
        pub struct #builder_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #ident {
            pub fn builder() -> #builder_name {
                #builder_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    })
}
