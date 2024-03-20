use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::ExprMatch;
use syn::Pat;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::visit_mut;
use syn::visit_mut::VisitMut;

#[derive(Debug)]
struct WrongLocations {
    target: String,
    expected: String,
    span: Span,
}

impl From<&WrongLocations> for syn::Error {
    fn from (wrong: &WrongLocations) -> Self{
        syn::Error::new(
            wrong.span,
            format!("{} should sort before {}", wrong.target, wrong.expected),
        )
    }
}

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    let item = parse_macro_input!(input as syn::Item);
    let syn::Item::Enum(item_enum) = item else {
        return syn::Error::new(Span::call_site(), "expected enum or match expression")
            .to_compile_error()
            .into()
    };
    let idents = item_enum
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .map(|ident| (ident.to_string(), ident.span()))
        .collect::<Vec<_>>();
    let mut wrong_positions = get_unsorted_items(&idents).iter()
        .map(|wrong| {
            syn::Error::new(
                wrong.span,
                format!("{} should sort before {}", wrong.target, wrong.expected),
            )
            .to_compile_error()
        })
        .collect::<TokenStream2>();
    wrong_positions.extend(item_enum.to_token_stream());
    wrong_positions.into()
}

struct SortedInFn(Vec<syn::Error>);

impl VisitMut for SortedInFn {
    fn visit_expr_match_mut (&mut self, expr_match: &mut ExprMatch) {
        let sorted_position = expr_match.attrs.iter().position(|v| {
            v.meta.path().is_ident("sorted")
        });
        if let Some(sorted_index) = sorted_position {
            expr_match.attrs.remove(sorted_index);

            fn path_to_str(path: &syn::Path) -> (String, Span) {
                let span = path.span();
                let str = path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
                (str, span)
            }

            let ident_spans = expr_match.arms.iter().map(|arm| {
                match &arm.pat {
                    Pat::TupleStruct(tuple_struct) => Ok(path_to_str(&tuple_struct.path)),
                    Pat::Ident(pat_ident) => Ok((pat_ident.ident.to_string(), pat_ident.ident.span())),
                    Pat::Wild(pat_wild) => Ok(("_".to_string(), pat_wild.span())),
                    _ => Err(syn::Error::new(arm.pat.span(), "unsupported by #[sorted]")),
                }
            }).collect::<Result<Vec<_>, _>>();
            let errors = match ident_spans {
                Ok(paths) => {
                    get_unsorted_items(&paths).iter().map(|wrong| wrong.into()).collect::<Vec<_>>()
                },
                Err(err) => vec![err],
            };

            self.0.extend(errors);
        }
        visit_mut::visit_expr_match_mut(self, expr_match);
    }
}

#[proc_macro_attribute]
pub fn check(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as syn::ItemFn);
    let mut sorted_in_fn = SortedInFn(Vec::new());
    sorted_in_fn.visit_item_fn_mut(&mut item);
    let mut errors = sorted_in_fn.0
        .iter()
        .map(|err| err.to_compile_error())
        .collect::<TokenStream2>();
    errors.extend(item.to_token_stream());
    errors.into()
}

fn get_unsorted_items(idents: &[(String, Span)]) -> Vec<WrongLocations> {
    let mut idents = idents.iter().enumerate().collect::<Vec<_>>();
    idents.sort_by_key(|v| &v.1.0);

    idents
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            idents
                .iter()
                .skip(i + 1)
                .find(|v| v.0 < item.0)
                .map(|other| WrongLocations {
                    target: item.1.0.to_string(),
                    expected: other.1.0.to_string(),
                    span: item.1.1,
                })
        })
        .collect::<Vec<_>>()
}
