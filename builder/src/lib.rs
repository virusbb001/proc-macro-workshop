use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Data, Type, PathArguments, GenericArgument};

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
            if is_option(ty) {
                quote! {
                    #ident: #ty
                }
            } else {
                quote! {
                    #ident: Option<#ty>
                }
            }
        })
    });

    let setters = fields.clone().filter_map(|field| {
        let ty = &field.ty;
        field.ident.as_ref().map(|ident| {
            if is_option(ty) {
                let arg_ty = get_type_in_generics(ty);

                quote! {
                    pub fn #ident(&mut self, #ident: #arg_ty) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }

            } else {
                quote! {
                    pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }
            }
        })
    });

    let field_guards = fields.clone().filter_map(|field| {
        let ty = &field.ty;
        field.ident.as_ref().map(|ident| {
            if is_option(ty) {
                quote! {
                    let current_dir = self.current_dir.clone();
                }
            } else {
                quote! {
                    let Some(#ident) = self.#ident.clone() else {
                        return Err("field is not enough".to_string().into());
                    };
                }
            }
        })
    });

    let field_idents = fields.clone().filter_map(|field| field.ident.as_ref());

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

        impl #builder_name {
            #(#setters)*

            pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
                #(#field_guards)*

                Ok(Command {
                    #(#field_idents),*
                })
            }
        }
    }.into()
}

fn is_option(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    type_path
        .path
        .segments
        .first()
        .map(|segment| segment.ident == "Option")
        .unwrap_or(false)
}

fn get_type_in_generics(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let PathArguments::AngleBracketed(ref args) = type_path.path.segments.first()?.arguments else {
        return None;
};
    let Some(GenericArgument::Type(ty)) = args.args.first() else {
        return None;
    };
    Some(ty)
}
