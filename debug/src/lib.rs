use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Expr, Fields, GenericParam,
    Generics, Lit, Meta, Type, TypePath, PathArguments, GenericArgument, MetaNameValue, ExprLit, WherePredicate,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(input_struct) = input.data else {
        return syn::Error::new(input.span(), "CustomDebug is not used for struct")
            .into_compile_error()
            .into();
    };
    let struct_bound = get_bound_in_debug_attr(&input.attrs).map(|bound| {
        let token_stream = bound.and_then(|bound| {
            syn::parse_str::<WherePredicate>(&bound).map(|predicate| predicate.to_token_stream())
        });

        match token_stream {
            Ok(token_stream) => token_stream,
            Err(err) => err.into_compile_error(),
        }
    }).into_iter().collect::<Vec<_>>();

    let ident = &input.ident;
    let ident_litstr = ident.to_string();

    let bound = if struct_bound.is_empty() {
        infer_bound_type_from_fields(&input_struct.fields, &input.generics)
    } else {
        struct_bound
    };
    let generics = if bound.is_empty() { add_trait_bounds(input.generics) } else { input.generics };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_call = input_struct.fields.iter().filter_map(|field| {
        let ident = field.ident.as_ref()?;
        let ident_str = ident.to_string();
        let debug_attr = get_debug_attr(&field.attrs);
        if let Some(debug_attr) = debug_attr {
            match debug_attr {
                Ok(debug) => Some(quote! {
                    .field(#ident_str, &format_args!(#debug, &self.#ident))
                }),
                Err(err) => Some(err.into_compile_error()),
            }
        } else {
            Some(quote! {
                .field(#ident_str, &self.#ident)
            })
        }
    });

    let impl_clause = if bound.is_empty() {
        quote! { #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause }
    } else {
        quote! { #impl_generics std::fmt::Debug for #ident #ty_generics where #(#bound),* }
    };
    quote! {
        impl #impl_clause {
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
        .find(|name_value| name_value.path.is_ident("debug"))
        .map(|name_value| {
            let Expr::Lit(ref lit) = name_value.value else {
                return Err(syn::Error::new(
                    name_value.span(),
                    "value of debug is not string",
                ));
            };
            match &lit.lit {
                Lit::Str(lit_str) => Ok(lit_str.value()),
                _ => Err(syn::Error::new(
                    lit.lit.span(),
                    "value of debug is not string",
                )),
            }
        })
}

fn get_type_phantom_data_in_fields(fields: &Fields) -> Vec<&Type> {
    fields
        .iter()
        .filter_map(|field| match field.ty {
            Type::Path(ref path)
                if path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "PhantomData") =>
            {
                Some(&field.ty)
            }
            _ => None,
        })
        .collect()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    generics
}

fn get_types_to_bind_debug<'a>(fields: &'a Fields, generics: &Generics) -> Vec<&'a TypePath> {
    let generics = generics.params.iter().filter_map(|param| {
        match param {
            GenericParam::Type(ty) => Some(&ty.ident),
            _ => None,
        }
    }).collect::<Vec<_>>();
    fields
        .iter()
        .filter_map(|field| {
            let Type::Path(ref ty) = field.ty else {
                return None;
            };
            Some(ty)
        })
        .filter_map(|ty| {
            let is_associated = ty.path.segments.first().is_some_and(|ty| {
                generics.contains(&&ty.ident)
            });
            if is_associated {
                Some(ty)
            } else {
                if ty.path.segments.first().is_some_and(|segment| {
                    segment.ident == "PhantomData"
                }) {
                    return None;
                }
                let Some(PathArguments::AngleBracketed(args)) = ty.path.segments.last().map(|segment| &segment.arguments) else {
                    return None;
                };
                args.args.iter().filter_map(|arg| {
                    if let GenericArgument::Type(ty) = arg {
                        Some(ty)
                    } else {
                        None
                    }
                }).filter_map(|ty| {
                    if let Type::Path(path) = ty {
                        Some(path)
                    } else {
                        None
                    }
                }).find(|ty| {
                        ty.path.segments.first().is_some_and(|ty| {
                            generics.contains(&&ty.ident)
                        })
                })
            }
        })
        .collect::<Vec<_>>()
}

fn get_bound_in_debug_attr(attrs: &[Attribute]) -> Option<Result<String, syn::Error>> {
    attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::List(attr_list) => Some(attr_list),
            _ => None,
        })
        .find(|list| list.path.is_ident("debug"))
        .and_then(|list| list.parse_args::<MetaNameValue>().ok())
        .filter(|name_value| name_value.path.is_ident("bound"))
        .map(|name_value| match name_value.value {
            Expr::Lit(ExprLit{lit: Lit::Str(litstr), ..}) => {
                Ok(litstr.value())
            }
            _ => Err(syn::Error::new(
                    name_value.span(),
                    "value of debug is not string",
                ))
        })
}

fn infer_bound_type_from_fields(
    fields: &Fields,
    generics: &Generics,
) -> Vec<TokenStream2>{
    let phantom_type_fields = get_type_phantom_data_in_fields(fields);
    let mut bound = phantom_type_fields
        .iter()
        .map(|ty| {
            quote! { #ty: std::fmt::Debug }
        })
        .collect::<Vec<_>>();

    bound.extend(
        get_types_to_bind_debug(
            fields,
            generics
        ).iter().map(|ty| {
            quote! { #ty: std::fmt::Debug }
        })
    );

    bound

}
