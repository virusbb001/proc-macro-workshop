use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;

    let field_tys = item_struct
        .fields
        .iter()
        .map(|field| &field.ty)
        .collect::<Vec<_>>();

    let field_bits = field_tys.iter().map(|ty| quote! { #ty::BITS }).collect::<Vec<_>>();

    let struct_define = quote! {
        #[repr(C)]
        pub struct #struct_name {
            data: [u8; (0 #(+ #field_tys::BITS)* )/8],
        }
    };

    let getter_and_setter = item_struct
        .fields
        .iter()
        .enumerate()
        .filter_map(|(i, field)| {
            let offset = field_tys.iter().skip(i + 1).map(|ty| {
                quote! { #ty::BITS }
            });

            let ty = &field.ty;
            let bits = quote! { #ty::BITS; };
            let offset = quote! { 0 #(+ #offset)* };

            let ident = field.ident.as_ref()?;
            let getter = Ident::new(&format!("get_{}", ident), Span::call_site());
            let setter = Ident::new(&format!("set_{}", ident), Span::call_site());
            Some(quote! {
                fn #getter (&self) -> u64 {
                    let bits = #bits;
                    let offset = #offset;

                    let data_len = self.data.len();

                    let arr_offset = offset / 8;
                    let bit_offset = u8::try_from(offset % 8).unwrap();
                    let mask_for_get_data = 2_u8.pow(bit_offset.into())-1;

                    create_bit_masks(bits, bit_offset)
                        .into_iter()
                        .rev()
                        .enumerate()
                        .filter(|(index, _)| index + arr_offset < data_len)
                        .map(|(index, mask)| {
                            let i = index + arr_offset;
                            self.data[i] & mask
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .scan(0, |curr, d| {
                            let c = *curr;
                            *curr = d & mask_for_get_data;
                            let data = d >> bit_offset | c << bit_offset;
                            Some(data)
                        })
                        .fold(0_u64, |decoded, d| decoded << 8 | u64::from(d))
                }

                fn #setter (&mut self, v: u64) {
                    let bits = #bits;
                    let offset = #offset;

                    let data_len = self.data.len();
                    let arr_offset = offset / 8;
                    let bit_offset = u8::try_from(offset % 8).unwrap();

                    // little endian
                    let value_bits = v
                        .to_le_bytes()
                        .into_iter()
                        .scan(0_u8, |cum, x| {
                            let left = *cum;
                            *cum = x >> bit_offset;
                            Some(x << bit_offset | left)
                        })
                        .collect::<Vec<_>>()
                    ;

                    create_bit_masks(bits, bit_offset)
                        .iter()
                        .rev()
                        .enumerate()
                        .filter(|(index, _)| index + arr_offset < data_len)
                        .for_each(|(index, mask)| {
                            let i = index + arr_offset;
                            self.data[i] = (self.data[i] & !mask)
                                | (mask & value_bits[index]);
                        });
                }
            })
        });

    let impl_defines = quote! {
        impl #struct_name {
            fn new() -> Self {
                Self {
                    data: Default::default(),
                }
            }
            #(#getter_and_setter)*
        }
    };

    quote! {
        #struct_define

        #impl_defines

        const _: self::checks::MultipleOfEight<[(); (0 #(+ #field_bits)* )% 8]> = ();
    }
    .into()
}
