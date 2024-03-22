use bitfield::*;
use std::fmt;

#[repr(C)]
pub struct TheirFourBytes {
    data: [u8; (B1::BITS + B3::BITS + B24::BITS + B4::BITS + B16::BITS) / 8],
}

impl fmt::Display for TheirFourBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data
            .iter()
            .rev()
            .try_for_each(|data| writeln!(f, "{:#010b}", data))
    }
}

impl TheirFourBytes {
    fn new() -> Self {
        Self {
            data: Default::default(),
        }
    }
    fn get_a(&self) -> u64 {
        let bits = B1::BITS;
        let offset = B3::BITS + B24::BITS + B4::BITS + B16::BITS;
        let data_len = self.data.len();

        let arr_offset = offset / 8;
        let bit_offset = u8::try_from(offset % 8).unwrap();
        let mask_for_get_data = 2_u8.pow(bit_offset.into()) - 1;

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
    fn set_a(&mut self, v: u64) {
        let bits = B1::BITS;
        let offset = B3::BITS + B24::BITS + B4::BITS + B16::BITS;

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
            .collect::<Vec<_>>();

        create_bit_masks(bits, bit_offset)
            .iter()
            .rev()
            .enumerate()
            .filter(|(index, _)| index + arr_offset < data_len)
            .for_each(|(index, mask)| {
                let i = index + arr_offset;
                self.data[i] = (self.data[i] & !mask) | (mask & value_bits[index]);
            });
    }
    fn get_b(&self) -> u64 {
        let bits = B3::BITS;
        let offset = B24::BITS + B4::BITS + B16::BITS;
        let data_len = self.data.len();

        let arr_offset = offset / 8;
        let bit_offset = u8::try_from(offset % 8).unwrap();
        let mask_for_get_data = 2_u8.pow(bit_offset.into()) - 1;

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

    fn get_c(&self) -> u64 {
        let bits = B24::BITS;
        let offset = B4::BITS + B16::BITS;
        let data_len = self.data.len();

        let arr_offset = offset / 8;
        let bit_offset = u8::try_from(offset % 8).unwrap();
        let mask_for_get_data = 2_u8.pow(bit_offset.into()) - 1;

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
    fn get_d(&self) -> u64 {
        let bits = B4::BITS;
        let offset = B16::BITS;
        let data_len = self.data.len();

        let arr_offset = offset / 8;
        let bit_offset = u8::try_from(offset % 8).unwrap();
        let mask_for_get_data = 2_u8.pow(bit_offset.into()) - 1;

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

    fn get_e(&self) -> u64 {
        let bits = B16::BITS;
        let offset = 0;
        let data_len = self.data.len();

        let arr_offset = offset / 8;
        let bit_offset = u8::try_from(offset % 8).unwrap();
        let mask_for_get_data = 2_u8.pow(bit_offset.into()) - 1;

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

    fn set_b(&mut self, v: u64) {
        let offset = B24::BITS + B4::BITS + B16::BITS;
        let bits = B3::BITS;

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
            .collect::<Vec<_>>();

        create_bit_masks(bits, bit_offset)
            .iter()
            .rev()
            .enumerate()
            .filter(|(index, _)| index + arr_offset < data_len)
            .for_each(|(index, mask)| {
                let i = index + arr_offset;
                self.data[i] = (self.data[i] & !mask) | (mask & value_bits[index]);
            });
    }

    fn set_c(&mut self, v: u64) {
        let bits = B24::BITS;
        let offset = B4::BITS + B16::BITS;

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
            .collect::<Vec<_>>();

        create_bit_masks(bits, bit_offset)
            .iter()
            .rev()
            .enumerate()
            .for_each(|(index, mask)| {
                let i = index + arr_offset;
                self.data[i] = (self.data[i] & !mask) | (mask & value_bits[index]);
            });
    }

    fn set_d(&mut self, v: u64) {}

    fn set_e(&mut self, v: u64) {
        let bits = B16::BITS;
        let offset = 0;

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
            .collect::<Vec<_>>();

        create_bit_masks(bits, bit_offset)
            .iter()
            .rev()
            .enumerate()
            .for_each(|(index, mask)| {
                let i = index + arr_offset;
                self.data[i] = (self.data[i] & !mask) | (mask & value_bits[index]);
            });
    }
}

fn main() {
    let mut bitfield = TheirFourBytes::new();
    bitfield.set_c(0b101010101010101010101010);
    eprintln!("{}", bitfield);
    bitfield.set_e(0b1010101010101010);
    assert_eq!(0b1010101010101010, bitfield.get_e());
    eprintln!("{}", bitfield);
    bitfield.set_a(0b1);
    eprintln!("{}", bitfield);
    bitfield.set_b(0b101);
    eprintln!("{}", bitfield);
    bitfield.set_a(0);
    bitfield.set_b(0);
    bitfield.set_e(0);

    assert_eq!(0b101010101010101010101010, bitfield.get_c());
    bitfield.set_c(0);

    assert_eq!(0, bitfield.get_a());
    assert_eq!(0, bitfield.get_b());
    assert_eq!(0, bitfield.get_c());
    assert_eq!(0, bitfield.get_d());

    bitfield.set_c(14);
    assert_eq!(0, bitfield.get_a());
    assert_eq!(0, bitfield.get_b());
    assert_eq!(14, bitfield.get_c());
    assert_eq!(0, bitfield.get_d());
}
