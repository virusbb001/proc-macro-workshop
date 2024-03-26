// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.

pub use bitfield_impl::bitfield;
pub use bitfield_impl::BitfieldSpecifier;
use seq::seq;
pub mod checks;

pub trait Specifier {
    const BITS: usize;
    type T;

    fn convert_to_u64(item: Self::T) -> u64;
    fn convert_from_u64(v: u64) -> Self::T;
}

impl Specifier for bool {
    const BITS: usize = 1;
    type T = bool;

    fn convert_to_u64(item: Self::T) -> u64 {
        match item {
            true => 1,
            false => 0,
        }
    }

    fn convert_from_u64(v: u64) -> Self::T {
        match v {
            0 => false,
            1 => true,
            _ => panic!("unexpected value: {}", v),
        }
    }
}

macro_rules! define_bit_enums {
    ($t: ty, $range: pat_param) => {
        seq!(N in $range {
            pub enum B~N {}

            impl Specifier for B~N {
                const BITS: usize = N;
                type T = $t;
                fn convert_to_u64(item: Self::T) -> u64 {
                    u64::from(item)
                }

                fn convert_from_u64(v: u64) -> Self::T {
                    Self::T::try_from(v).unwrap()
                }
            }
        });
    }
}

define_bit_enums!(u8, 0..=8);
define_bit_enums!(u16, 9..=16);
define_bit_enums!(u32, 17..=32);
define_bit_enums!(u64, 33..=64);

/**
* field_size: length of bits
* offset: offset of bitfields. It should be less than 8
*
* big endian
*/
pub fn create_bit_masks(field_size: usize, offset: u8) -> Vec<u8> {
    let high_bit = field_size + usize::from(offset);
    let size = high_bit / 8;
    let mut fields = (0..=size).map(|_| !0_u8).collect::<Vec<_>>();
    let first_mask = !(!0 << (high_bit % 8));
    let last_mask = !0_u8 << offset;
    if let Some(v) = fields.first_mut() {
        *v = first_mask;
    }
    if let Some(v) = fields.last_mut() {
        *v &= last_mask;
    }

    fields
}
pub fn create_value_bits(v: u64, bit_offset: u8) -> Vec<u8> {
    v.to_le_bytes()
        .into_iter()
        .scan(0_u8, |cum, x| {
            let left = *cum;
            *cum = x.checked_shr((8 - bit_offset).into()).unwrap_or(0);
            Some(x << bit_offset | left)
        })
        .collect::<Vec<_>>()
}

pub fn create_value_from_le_bytes (bytes: &[u8], offset: u8) -> u64 {
    bytes.iter()
        .rev()
        .fold(0_u64, |cum, x| {
                cum << 8 | u64::from(*x)
        }) >> offset
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bits(left: u64, right: u64) {
        assert_eq!(left, right, "\n left: {:#10b}\nright: {:#10b}", left, right);
    }

    #[test]
    fn test_create_bit_masks() {
        let result = create_bit_masks(8, 4);
        assert_eq!(result, vec![0b00001111, 0b11110000]);
        create_bit_masks(1, 0);
        create_bit_masks(2, 0);
        create_bit_masks(2, 1);
        create_bit_masks(2, 2);
        let result = create_bit_masks(1, 0);
        assert_eq!(result, vec![0b00000001]);

        let result = create_bit_masks(1, 1);
        assert_eq!(result, vec![0b00000010]);

        let result = create_bit_masks(3, 3);
        assert_eq!(result, vec![0b00111000]);

        assert_eq!(
        create_bit_masks(13, 0),
        vec![
            0b00011111,
            0b11111111,
        ]
        );
    }

    #[test]
    fn test_create_value_bits() {
        assert_eq!(create_value_bits(1, 0), vec![1, 0, 0, 0, 0, 0, 0 ,0]);
        assert_eq!(create_value_bits(1, 1), vec![0b10, 0, 0, 0, 0, 0, 0 ,0]);
    }

    #[test]
    fn test_create_value_from_le_bytes () {
        assert_eq!(create_value_from_le_bytes(&[
            0b00000000
        ], 0), 0);

        assert_eq!(create_value_from_le_bytes(&[
            0b00000001
        ], 0), 1);
        assert_bits(create_value_from_le_bytes(&[
            0b10000001,
            0b00000001
        ], 0), 0b110000001);

        assert_bits(create_value_from_le_bytes(&[
            0b00000010,
            0b00000011
        ], 1), 0b110000001);
    }
}
