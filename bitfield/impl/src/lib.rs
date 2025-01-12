use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Ident, ItemStruct, Meta};

#[proc_macro_derive(BitfieldSpecifier)]
pub fn derive_bitfield_specifier(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let Data::Enum(data_enum) = derive_input.data else {
        return syn::Error::new(Span::call_site(), "BitfieldSpecifier can only use for enum").into_compile_error().into();
    };
    let variants_number = data_enum.variants.len();
    if !variants_number.is_power_of_two() {
        return syn::Error::new(Span::call_site(), "BitfieldSpecifier expected a number of variants which is a power of 2").into_compile_error().into();
    }
    let bits = variants_number.ilog2();
    let bn = Ident::new(&format!("B{}", bits), Span::call_site());
    let ident = &derive_input.ident;
    let match_arms = data_enum.variants.iter().map(|variant| {
        let name = &variant.ident;
        quote! {
            _ if #ident::#name as u64 == item => #ident::#name
        }
    });

    let arr_list = data_enum.variants.iter().map(|variant| {
        let name = &variant.ident;
        quote! {
            #ident::#name as usize
        }
    });
    let check_discriminant_range = quote! {
        const _: () = {
            let mut max = 0_usize;
            let a = [#(#arr_list,)*];
            let mut i = 0;
            while i < a.len() {
                max = if max < a[i] {
                    a[i]
                } else {
                    max
                };
                i+=1;
            }

            let bits: usize = 1 << #ident::BITS;

            if max >= bits {
                panic!("max discriminant is out of range");
            }
        };
    };
    quote! {
        impl Specifier for #ident {
            const BITS:usize = #bn::BITS;
            type T = #ident;

            fn convert_to_u64(item: Self::T) -> u64 {
                item as u64
            }

            fn convert_from_u64(item: u64) -> Self::T {
                match item {
                    #(#match_arms,)*
                    _ => panic!("unexpected value: {}", item),
                }
            }
        }

        #check_discriminant_range
    }.into()
}

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
            let field_type = quote! { <#ty as Specifier>::T };
            let getter = Ident::new(&format!("get_{}", ident), Span::call_site());
            let setter = Ident::new(&format!("set_{}", ident), Span::call_site());
            Some(quote! {
                fn #getter (&self) -> #field_type {
                    let bits = #bits;
                    let offset = #offset;

                    let data_len = self.data.len();

                    let arr_offset = offset / 8;
                    let bit_offset = u8::try_from(offset % 8).unwrap();
                    let mask_for_get_data = 2_u8.pow(bit_offset.into())-1;

                    let v = create_bit_masks(bits, bit_offset)
                        .into_iter()
                        .rev()
                        .enumerate()
                        .filter(|(index, _)| index + arr_offset < data_len)
                        .map(|(index, mask)| {
                            let i = index + arr_offset;
                            self.data[i] & mask
                        })
                        .collect::<Vec<_>>()
                        ;
                    let v = create_value_from_le_bytes(&v, bit_offset);
                    
                    <#ty as Specifier>::convert_from_u64(v)
                }

                fn #setter (&mut self, v: #field_type) {
                    let bits = #bits;
                    let offset = #offset;

                    let data_len = self.data.len();
                    let arr_offset = offset / 8;
                    let bit_offset = u8::try_from(offset % 8).unwrap();
                    let v = <#ty as Specifier>::convert_to_u64(v);

                    // little endian
                    let value_bits = create_value_bits(v, bit_offset);

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

    // check bits attribute
    let check_bits = item_struct
        .fields
        .iter()
        .filter_map(|field| {
            let ty = &field.ty;
            let bit_attributes = field.attrs.iter().find_map(|attr| {
                let Meta::NameValue(name_value) = &attr.meta else {
                    return None;
                };
                if !(name_value.path.is_ident("bits")) {
                    return None;
                }
                Some(&name_value.value)
            })?;
            let attr_span = bit_attributes.span();

            Some(quote_spanned! { attr_span =>
                const _: [(); #bit_attributes] = [(); <#ty as Specifier>::BITS];
            })
        })
    ;

    quote! {
        #struct_define

        #impl_defines

        impl std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.data.iter().enumerate().rev().try_for_each(|(i, d)| {
                    f.write_fmt(format_args!("{:#2}: {:#010b}\n", i, d))
                })
            }
        }

        #(#check_bits)*
        const _: self::checks::MultipleOfEight<[(); (0 #(+ #field_bits)* )% 8]> = ();
    }
    .into()
}
