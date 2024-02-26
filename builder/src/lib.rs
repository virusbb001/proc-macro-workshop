use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Field, Fields, GenericArgument, LitStr, MetaNameValue,
    PathArguments, Type, Expr, Lit, Attribute,
};

#[proc_macro_derive(Builder, attributes(builder))]
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
        let Field { vis, ident, ty, attrs, .. } = v;
        let field_type = if is_option(ty) || get_each_arg(attrs).is_some() {
            quote! { #ty }
        } else {
            quote! { Option<#ty> }
        };
        quote! {
            #vis #ident: #field_type
        }
    });

    let build_struct_init_fields = fields.named.iter().map(|v| {
        let init = if is_vec(&v.ty) && get_each_arg(&v.attrs).is_some() {
            quote! { Vec::new() }
        } else {
            quote! { None }
        };
        let ident = &v.ident;

        quote! {
            #ident: #init
        }
    });

    let setter_fns = fields
        .named
        .iter()
        .filter_map(|v| {
            let Field {
                ident, ty, attrs, ..
            } = v;

            if is_option(ty) {
                get_type_of_option(ty).map(|ty| (ident, ty, None))
            } else {
                Some((ident, ty, get_each_arg(attrs)))
            }
        })
        .map(|(ident, ty, each)| {
            if let Some(each) = each {
                if !is_vec(ty) {
                    panic!("each should be used for Vec<_>");
                }
                let ty = get_generics_of_type(ty);
                let each = syn::Ident::new(&each.value(), Span::call_site());
                quote! {
                    fn #each(&mut self, #each: #ty) -> &mut Self {
                        self.#ident.push(#each);
                        self
                    }
                }
            } else {
                quote! {
                    fn #ident(&mut self, #ident: #ty) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }
            }
        });

    let builder_guards = fields.named.iter().filter_map(|v| {
        let Some(ident) = &v.ident else {
            return None;
        };

        if is_option(&v.ty) || (is_vec(&v.ty) && get_each_arg(&v.attrs).is_some()){
            Some(quote! {
                let #ident = self.#ident.clone();
            })
        } else {
            let error_message =
                LitStr::new(&format!("field {} is missing", ident), Span::call_site());

            Some(quote! {
                let #ident = self.#ident.clone().ok_or(#error_message)?;
            })
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

fn is_option(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    let Some(type_path) = type_path.path.segments.first() else {
        return false;
    };

    type_path.ident == "Option"
}

fn is_vec(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    let Some(type_path) = type_path.path.segments.first() else {
        return false;
    };

    type_path.ident == "Vec"
}

fn get_generics_of_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let type_path = type_path.path.segments.first()?;

    let PathArguments::AngleBracketed(ref generic_args) = type_path.arguments else {
        return None;
    };

    generic_args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(generic_type) => Some(generic_type),
        _ => None,
    })
}

fn get_type_of_option(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let Some(type_path) = type_path.path.segments.first() else {
        return None;
    };

    if type_path.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(ref generic_args) = type_path.arguments else {
        return None;
    };

    generic_args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(generic_type) => Some(generic_type),
        _ => None,
    })
}

fn get_each_arg(attrs: &[Attribute]) -> Option<LitStr> {
    let each = attrs
        .iter()
        .filter_map(|attr| attr.parse_args::<MetaNameValue>().ok())
        .find(|attr| {
            attr.path
                .segments
                .iter()
                .any(|segment| segment.ident == "each")
        })?;

    let Expr::Lit(each) = each.value else {
        panic!("value of each should be string literal");
    };
    let Lit::Str(each) = each.lit else {
        panic!("value of each should be string literal");
    };

    Some(each)
}
