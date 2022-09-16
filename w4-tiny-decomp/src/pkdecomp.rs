use tiny_bitfiddle::{BitReader, BitSliceWriter, BitWriter};
use w4_pnger_common::BitsPerPixel;

use super::*;

pub fn decompress<'a>(
    decompressor: &'a mut Decompressor,
    bytes: &[u8],
) -> Result<SpriteHandle<'a>, &'static str> {
    let width = bytes[0] as usize;
    let height = bytes[1] as usize;
    let flags = bytes[2];
    let split = bytes[3] & (1 << 0) == 1;
    let xor = bytes[3] & (1 << 1) != 0;
    let seq_delta = (bytes[3] & 0b00111100) >> 2;
    let jump_delta = (bytes[3] & 0b11000000) >> 6;
    let jump_size = bytes[4];

    let mut writer = BitSliceWriter::new(decompressor.buf);
    let mut reader = BitReader::new(&bytes[5..]);

    let mut state = if reader.read_bit().unwrap() {
        PkDecompressorState::StartVerbatim
    } else {
        PkDecompressorState::Rle
    };

    loop {
        match state {
            PkDecompressorState::StartVerbatim => match reader.read_bit() {
                Some(more_than_one) => {
                    if !more_than_one {
                        let kind = match reader.read_bit() {
                            Some(b) => b,
                            None => break,
                        };

                        match kind {
                            true => {
                                writer.write_bit(1);
                                writer.write_bit(0);
                            }
                            false => {
                                writer.write_bit(0);
                                writer.write_bit(1);
                            }
                        }

                        state = PkDecompressorState::Rle;
                    } else {
                        state = PkDecompressorState::Verbatim;
                    }
                }
                None => break,
            },
            PkDecompressorState::Verbatim => {
                let b1 = match reader.read_bit() {
                    Some(b) => b,
                    None => break,
                };
                let b2 = match reader.read_bit() {
                    Some(b) => b,
                    None => break,
                };

                if b1 == false && b2 == false {
                    state = PkDecompressorState::Rle;
                } else {
                    writer.write_bit(b1 as u8);
                    writer.write_bit(b2 as u8);
                }
            }
            PkDecompressorState::Rle => {
                let mut len = 0;
                let mut bits = 0;

                let mut front_bit = match reader.read_bit() {
                    Some(b) => b,
                    None => break,
                };
                while front_bit {
                    len += 1;
                    bits = bits << 1 | 1;

                    front_bit = match reader.read_bit() {
                        Some(b) => b,
                        None => break,
                    }
                }

                len += 1;
                bits <<= 1;

                let mut bits_2 = 0;
                for _ in 0..len {
                    let b = match reader.read_bit() {
                        Some(b) => b,
                        None => break,
                    };
                    bits_2 = bits_2 << 1 | b as u8;
                }

                let sum = bits + bits_2 + 1;
                for _ in 0..sum {
                    writer.write_bit(0);
                    writer.write_bit(0);
                }
                state = PkDecompressorState::StartVerbatim;
            }
        }
    }
    
    let byte_end = match BitsPerPixel::try_from_flags(flags)? {
        BitsPerPixel::One => (width * height) / 8,
        BitsPerPixel::Two => (width * height) / 4,
    };

    let mut written_bytes = &mut decompressor.buf[..byte_end];

    for _ in 0..jump_delta {
        jump_delta_decode_in_place(&mut written_bytes, jump_size as usize);
    }

    for _ in 0..seq_delta {
        delta_decode_in_place(&mut written_bytes);
    }

    if xor {
        debug_assert!(written_bytes.len() % 2 == 0);
        let (left, right) = written_bytes.split_at_mut(written_bytes.len() / 2);
        xor_bitplanes(right, left);
    }

    if split {
        assemble_bitplanes_in_place(&mut written_bytes);
    }

    Ok(SpriteHandle {
        bytes: written_bytes,
        width: width as u8,
        height: height as u8,
        flags: flags as u8,
    })
}

#[derive(Debug)]
enum PkDecompressorState {
    StartVerbatim,
    Verbatim,
    Rle,
}
