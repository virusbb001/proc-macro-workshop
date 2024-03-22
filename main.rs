// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use bitfield::*;

#[bitfield]
pub struct MyFourBytes {
    a: B1,
    b: B3,
    c: B4,
    d: B24,
}

fn main() {
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
