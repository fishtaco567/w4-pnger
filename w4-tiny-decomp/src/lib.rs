#![cfg_attr(not(test), no_std)]

mod pkdecomp;
use core::convert::TryInto;

pub use pkdecomp::*;

use tiny_bitfiddle::{BitSliceWriter, BitWriter};
use w4_pnger_common::{BitsPerPixel, CompType};

pub struct Decompressor<'a> {
    pub(crate) buf: &'a mut [u8],
}

impl<'a> Decompressor<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf }
    }

    pub fn decompress(&mut self, bytes: &[u8]) -> Result<SpriteHandle, &'static str> {
        let kind: CompType = bytes[0].try_into()?;

        match kind {
            CompType::Uncompressed => {
                let width = bytes[1];
                let height = bytes[2];
                let flags = bytes[3];

                let length = match BitsPerPixel::try_from_flags(flags)? {
                    BitsPerPixel::One => (width as usize * height as usize) / 8,
                    BitsPerPixel::Two => (width as usize * height as usize) / 4,
                };

                Ok(SpriteHandle {
                    bytes: &self.buf,
                    width,
                    height,
                    flags,
                })
            }
            CompType::Pk => pkdecomp::decompress(self, &bytes[1..]),
        }
    }
}

pub struct SpriteHandle<'a> {
    pub bytes: &'a [u8],
    pub width: u8,
    pub height: u8,
    pub flags: u8,
}

pub(crate) fn xor_bitplanes(bp1: &[u8], bp2: &mut [u8]) {
    for (b1, b2) in bp1.iter().zip(bp2.iter_mut()) {
        *b2 = *b1 ^ *b2;
    }
}

pub(crate) fn assemble_bitplanes_in_place(bytes: &mut [u8]) {
    in_shuffle(bytes);
}

pub(crate) fn delta_decode_in_place(bytes: &mut [u8]) {
    let len = bytes.len() * 8;

    let mut writer = BitSliceWriter::new(bytes);

    let mut cur = 0;

    for bit_pos in 0..len {
        let b1 = writer.read_at(bit_pos).unwrap(); //Guarenteed by the loop range

        if b1 {
            cur = 1 - cur;
        }

        writer.write_bit(cur);
    }
}

pub(crate) fn jump_delta_decode_in_place(bytes: &mut [u8], jump_size: usize) {
    let len = bytes.len() * 8;

    let mut writer = BitSliceWriter::new(bytes);

    for bit_pos in jump_size..len {
        let b_jump = writer.read_at(bit_pos - jump_size).unwrap(); //Guarenteed by the loop range

        let b_cur = writer.read_at(bit_pos).unwrap(); //Guarenteed by the loop range

        writer.write_bit_at((b_jump as u8 + b_cur as u8) % 2, bit_pos);
    }
}

fn in_shuffle_sides(bytes: &mut [u8], start: usize, len: usize) {
    let mut writer = BitSliceWriter::new(bytes);

    let size = len;
    let n = size / 2;
    let mut i = 1;
    if size == 0 || (size & 1 == 1) {
        return;
    }
    while i * 3 <= size + 1 {
        i *= 3; // Largest power of three
    }
    let m = (i - 1) / 2;

    writer.rotate_right(m + start, m + n + start, m);

    let mut m = 1;
    while m < i - 1 {
        // Permutation cycles
        let idx = start + (m * 2) % i - 1;

        let mut tmp1 = writer.read_at(idx).unwrap();
        writer.write_bit_at(writer.read_at(start + m - 1).unwrap() as u8, idx);

        let mut j = (m * 2) % i;
        while j != m {
            let idx = start + (j * 2) % i - 1;
            let tmp2 = writer.read_at(idx).unwrap();
            writer.write_bit_at(tmp1 as u8, idx);
            tmp1 = tmp2;
            j = (j * 2) % i;
        }

        m *= 3;
    }

    in_shuffle_sides(bytes, start + i - 1, len - (i - 1)); // Split and process the remaining elements
}

fn in_shuffle(slice: &mut [u8]) {
    in_shuffle_sides(slice, 1, slice.len() * 8 - 2); // Exclude the first and last elements
}

#[cfg(test)]
mod tests {
    use crate::{
        assemble_bitplanes_in_place, delta_decode_in_place, jump_delta_decode_in_place,
        xor_bitplanes,
    };
    use tiny_bitfiddle::{BitReader, BitSliceWriter, BitWriter};

    #[test]
    fn test_xor_bitplanes() {
        let mut bp1 = vec![0b11001100, 0b00110011u8];
        let mut bp2 = vec![0b10101010, 0b00001111u8];
        let mut bpr = vec![0b01100110, 0b00111100u8];

        xor_bitplanes(&bp1[..], &mut bp2[..]);
        assert_eq!(bp2, bpr);
    }

    #[test]
    fn test_rotate() {
        let mut bytes = vec![0b11001100, 0b00110011u8];
        let mut outtt = vec![0b00111100, 0b00110011u8];
        let mut writer = BitSliceWriter::new(&mut bytes[..]);

        writer.rotate_right(4, 8, 2);
        assert_eq!(bytes, outtt);
    }

    #[test]
    fn test_assemble_bitplanes() {
        let mut bp1 = vec![0b11001100, 0b11001100, 0b00110011, 0b00110011u8];
        let mut bpr = vec![0b01011010, 0b01011010, 0b01011010, 0b01011010u8];

        assemble_bitplanes_in_place(&mut bp1[..]);
        assert_eq!(bp1, bpr);
    }

    #[test]
    fn test_delta_decode_in_place() {
        let mut bytes = vec![0b00000001, 0b00100100, 0b00001000u8];
        let mut resul = vec![0b11111111, 0b11100011, 0b00000111u8];

        delta_decode_in_place(&mut bytes[..]);
        assert_eq!(bytes, resul);
    }

    #[test]
    fn test_jump_decode_in_place() {
        let mut bytes = vec![0b00111100, 0b00110100, 0b10110101u8];
        let mut resul = vec![0b00111100, 0b00001000, 0b10111101u8];

        jump_delta_decode_in_place(&mut bytes[..], 8);
        assert_eq!(bytes, resul);

        let mut bytes = vec![0b00100011, 0b00100100, 0b10000001u8];
        let mut resul = vec![0b00010011, 0b01110101, 0b11100110u8];

        jump_delta_decode_in_place(&mut bytes[..], 4);
        assert_eq!(bytes, resul);
    }
}
