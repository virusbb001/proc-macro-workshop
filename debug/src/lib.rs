use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Meta, Attribute, Expr, Lit};
use syn::spanned::Spanned;

#[proc_macro_derive(CustomDebug, attributes(debug))]
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
        let debug_attr = get_debug_attr(&field.attrs);
        if let Some(debug_attr) = debug_attr {
            match debug_attr {
                Ok(debug) => {
                    Some(quote! {
                        .field(#ident_str, &format_args!(#debug, &self.#ident))
                    })
                },
                Err(err) => Some(err.into_compile_error()),
            }
        } else {
            Some(quote! {
                .field(#ident_str, &self.#ident)
            })
        }
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

fn get_debug_attr(attrs: &[Attribute]) -> Option<Result<String, syn::Error>> {
    attrs
        .iter()
        .filter_map(|attr| {
            match &attr.meta {
                Meta::NameValue(name_value) => Some(name_value),
                _ => None,
            }
        })
        .find(|name_value| {
            name_value.path.is_ident("debug")
        })
        .map(|name_value| {
            let Expr::Lit(ref lit) = name_value.value else {
                return Err(syn::Error::new(name_value.span(), "value of debug is not string"));
            };
            match &lit.lit {
                Lit::Str(lit_str) => Ok(lit_str.value()),
                _ => Err(syn::Error::new(lit.lit.span(), "value of debug is not string"))
            }
        })
}
