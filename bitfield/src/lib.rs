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
use seq::seq;
pub mod checks;

pub trait Specifier {
    const BITS: usize;
    type T;
}

macro_rules! define_bit_enums {
    ($t: ty, $range: pat_param) => {
        seq!(N in $range {
            pub enum B~N {}

            impl Specifier for B~N {
                const BITS: usize = N;
                type T = $t;
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
    let mask_width = if fields.len() == 1 {
        offset + u8::try_from(field_size).unwrap()
    } else {
        offset
    };
    let mask = 2_u8.pow(mask_width.into()) - 1;
    if let Some(v) = fields.first_mut() {
        *v = mask;
    }
    if let Some(v) = fields.last_mut() {
        *v &= !0_u8 << offset;
    }

    fields
}
#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
