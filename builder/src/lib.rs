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
    let Fields::Named(fields) = data.fields else {
        panic!("Builder is not used for struct with named fields");
    };
    let build_struct_type_fields = fields.named.iter().map(|v| {
        let vis = &v.vis;
        let ident = &v.ident;
        let ty = &v.ty;
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
    })
}
