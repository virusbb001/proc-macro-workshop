use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let item_struct = parse_macro_input!(input as ItemStruct);

    let field_tys = item_struct.fields.iter().map(|field| &field.ty);

    quote! {
        #[repr(C)]
        pub struct MyFourBytes {
            data: [u8; (0 #(+ #field_tys::BITS)* )/8],
        }
    }.into()
}
