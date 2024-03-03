use syn::spanned::Spanned;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, Ident, Data, Type, PathArguments, GenericArgument, Attribute, MetaNameValue, Expr, Lit, parse_macro_input};

#[proc_macro_derive(Builder, attributes(builder))]
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

    let unexpected_attrs = fields.clone().find_map(|field|
        get_unexpected_attributes(&field.attrs).map(|err| err.to_compile_error())
    );

    let builder_init = fields.clone().filter_map(|field| {
        let attrs = &field.attrs;
        field.ident.as_ref().map(|ident| {
            if get_value_of_each(attrs).is_some() {
                quote! {
                    #ident: Vec::new()
                }
            } else {
                quote! {
                    #ident: None
                }
            }
        })
    });

    let builder_field = fields.clone().filter_map(|field| {
        let ty = &field.ty;
        field.ident.as_ref().map(|ident| {
            if is_option(ty) || get_value_of_each(&field.attrs).is_some() {
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
        let attrs = &field.attrs;
        field.ident.as_ref().map(|ident| {
            if is_option(ty) {
                let arg_ty = get_type_in_generics(ty);

                quote! {
                    pub fn #ident(&mut self, #ident: #arg_ty) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }

            } else if let Some(each) = get_value_of_each(attrs) {
                let each = Ident::new(&each, Span::call_site());
                let arg_ty = get_type_in_generics(ty);
                quote! {
                    pub fn #each(&mut self, #each: #arg_ty) -> &mut Self {
                        self.#ident.push(#each);
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
        let attrs = &field.attrs;
        field.ident.as_ref().map(|ident| {
            if is_option(ty) || get_value_of_each(attrs).is_some() {
                quote! {
                    let #ident = self.#ident.clone();
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
        #unexpected_attrs

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

fn get_value_of_each(attrs: &[Attribute]) -> Option<String> {
    attrs
        .first()
        .and_then(|attr| attr.parse_args::<MetaNameValue>().ok())
        .filter(|name_value| name_value.path.is_ident("each"))
        .and_then(|name_value| {
            let Expr::Lit(lit) = name_value.value else {
                return None;
            };

            let Lit::Str(lit_str) = lit.lit else {
                return None;
            };

            Some(lit_str.value())
        })
}

fn get_unexpected_attributes(attrs: &[Attribute]) -> Option<syn::Error> {
    attrs
        .first()
        .filter(|attr|
            attr.parse_args::<MetaNameValue>()
                .ok()
                .is_some_and(|name_value| !name_value.path.is_ident("each")))
        .map(|attr| syn::Error::new(attr.meta.span(), "expected `builder(each = \"...\")`"))
}
