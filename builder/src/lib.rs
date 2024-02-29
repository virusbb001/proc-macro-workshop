use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Data};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = Ident::new(&format!("{}Builder", name), Span::call_site());

    let Data::Struct(data_struct) = input.data else {
        panic!("builder is not used for struct");
    };

    let fields = data_struct
        .fields
        .iter()
        .filter(|field| field.ident.is_some());

    let builder_init = fields.clone().filter_map(|field| {
        field.ident.as_ref().map(|ident| {
            quote! {
                #ident: None
            }
        })
    });

    let builder_field = fields.clone().filter_map(|field| {
        let ty = &field.ty;
        field.ident.as_ref().map(|ident| {
            quote! {
                #ident: Option<#ty>
            }
        })
    });

    quote! {
        pub struct #builder_name {
            #(#builder_field),*
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_init),*
                }
            }
        }
    }.into()
}
