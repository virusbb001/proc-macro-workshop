use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, LitStr};
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;
    let Data::Struct(data) = ast.data else {
        panic!("Builder is not used for struct");
    };
    let Fields::Named(fields) = data.fields else {
        panic!("Builder is not used for struct with named fields");
    };
    let build_struct_type_fields = fields.named.iter().map(|v| {
        let Field { vis, ident, ty, .. } = v;
        quote! {
            #vis #ident: Option<#ty>
        }
    });

    let build_struct_init_fields = fields.named.iter().map(|v| {
        let ident = &v.ident;

        quote! {
            #ident: None
        }
    });

    let setter_fns = fields.named.iter().map(|v| {
        let Field { ident, ty, .. } = v;

        quote! {
            fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = Some(#ident);
                self
            }
        }
    });

    let builder_guards = fields.named.iter().map(|v| {
        let Some(ident) = &v.ident else {
            return quote! {}
        };

        let error_message = LitStr::new(&format!("field {} is missing", ident), Span::call_site());

        quote! {
            let #ident = self.#ident.clone().ok_or(#error_message)?;
        }
    });

    let builder_fields = fields.named.iter().map(|v| &v.ident);

    let builder_name = syn::Ident::new(&format!("{}Builder", ident), Span::call_site());
    proc_macro::TokenStream::from(quote! {
        pub struct #builder_name {
            #(#build_struct_type_fields),*
        }

        impl #ident {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#build_struct_init_fields),*
                }
            }
        }

        impl #builder_name {
            #(#setter_fns)*

            pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
                #(#builder_guards)*
                Ok(Command {
                    #(#builder_fields),*
                })
            }
        }
    })
}
