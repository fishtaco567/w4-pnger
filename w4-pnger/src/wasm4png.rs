use std::{collections::HashMap, convert::TryInto, fs::File, hash::Hash};

use anyhow::{bail, Result};
use png::{BitDepth, ColorType, OutputInfo, Reader};
use w4_pnger_common::BitsPerPixel;

pub struct W4Sprite {
    bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub bpp: BitsPerPixel,
}

impl W4Sprite {
    pub fn from_reader(reader: &mut Reader<File>) -> Result<Self> {
        let mut buf = vec![0; reader.output_buffer_size()];

        let info = reader.next_frame(&mut buf)?;

        let png = &buf[..info.buffer_size()];

        let (palette, bpp, components) = match info.color_type {
            png::ColorType::Grayscale => get_palette_bpp(&info, &png, 1).unwrap(),
            png::ColorType::Rgb => get_palette_bpp(&info, &png, 3).unwrap(),
            png::ColorType::Indexed => {
                let mut palette = HashMap::new();

                let bytes_per_component = bit_depth_to_bytes(info.bit_depth);

                for y in 0..info.height {
                    for x in 0..info.width {
                        let idx = bytes_per_component * (y * info.width + x) as usize;

                        if !palette.contains_key(&png[idx..idx + bytes_per_component]) {
                            palette.insert(
                                &png[idx..idx + bytes_per_component],
                                assume_u8(&png[idx..idx + bytes_per_component], info.bit_depth)
                                    as usize,
                            );
                        }
                    }
                }

                let bpp = if palette.len() > 2 {
                    BitsPerPixel::Two
                } else {
                    BitsPerPixel::One
                };

                (palette, bpp, 1)
            }
            png::ColorType::GrayscaleAlpha => get_palette_bpp(&info, &png, 2).unwrap(),
            png::ColorType::Rgba => get_palette_bpp(&info, &png, 4).unwrap(),
        };
        //Reimplementation of Aduros' png packing in WASM-4

        let mut out_bytes: Vec<u8> =
            vec![0; (info.width * info.height * bpp.get_num() / 8) as usize];

        let bytes_per_coponent = bit_depth_to_bytes(info.bit_depth);

        for y in 0..info.height {
            for x in 0..info.width {
                let idx = bytes_per_coponent * components * (info.width * y + x) as usize;
                let palette_index = *palette
                    .get(&png[idx..idx + bytes_per_coponent * components])
                    .unwrap() as u8;

                let (out_idx, shift, mask) = match bpp {
                    BitsPerPixel::One => {
                        let out_idx = ((y * info.width + x) >> 3) as usize;
                        let shift = 7 - (x & 0x7);
                        let mask = 0x1 << shift;
                        (out_idx, shift, mask)
                    }
                    BitsPerPixel::Two => {
                        let out_idx = ((y * info.width + x) >> 2) as usize;
                        let shift = 6 - ((x & 0x3) << 1);
                        let mask = 0x3 << shift;
                        (out_idx, shift, mask)
                    }
                };

                out_bytes[out_idx] = (palette_index << shift) | (out_bytes[out_idx] & (!mask));
            }
        }

        Ok(Self {
            bytes: out_bytes,
            width: info.width.try_into()?,
            height: info.height.try_into()?,
            bpp,
        })
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.bytes.len());
        out.extend_from_slice(&self.bytes);

        out
    }

    pub fn get_header_bytes(&self) -> Vec<u8> {
        vec![self.width as u8, self.height as u8, self.bpp.get_flags()]
    }
}

fn get_palette_bpp<'a>(
    info: &OutputInfo,
    png: &'a [u8],
    components: usize,
) -> Result<(HashMap<&'a [u8], usize>, BitsPerPixel, usize)> {
    let mut palette = HashMap::new();
    let bytes_per_component = bit_depth_to_bytes(info.bit_depth);
    for y in 0..info.height {
        for x in 0..info.width {
            let idx = bytes_per_component * components * (y * info.width + x) as usize;

            let slice = &png[idx..idx + bytes_per_component * components];

            if !palette.contains_key(slice) {
                if palette.len() > 4 {
                    bail!(
                        "Too many colors, first instance of fifth color found at {}, {}",
                        x,
                        y
                    )
                }

                palette.insert(slice, palette.len());
            }
        }
    }

    let mut palette_keys: Vec<&[u8]> = palette.keys().map(|x| x.clone()).collect();
    palette_keys.sort_by(|c1, c2| {
        Color::from_slice(c2, info.bit_depth, info.color_type)
            .unwrap()
            .brightness()
            .total_cmp(
                &Color::from_slice(c1, info.bit_depth, info.color_type)
                    .unwrap()
                    .brightness(),
            )
    });

    for (i, color) in palette_keys.iter().enumerate() {
        *palette.get_mut(color).unwrap() = i;
    }

    let color_count = palette_keys.len();

    let bpp = if color_count <= 2 {
        BitsPerPixel::One
    } else {
        BitsPerPixel::Two
    };
    Ok((palette, bpp, components))
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn from_slice(slice: &[u8], bit_depth: BitDepth, color_type: ColorType) -> Option<Self> {
        let bytes_per_component = bit_depth_to_bytes(bit_depth);

        match color_type {
            ColorType::Grayscale => {
                let g = compress_to_u8(&slice[0..bytes_per_component], bit_depth);
                Some(Self {
                    r: g,
                    g: g,
                    b: g,
                    a: 1,
                })
            }
            ColorType::Rgb => Some(Self {
                r: compress_to_u8(&slice[0..bytes_per_component], bit_depth),
                g: compress_to_u8(
                    &slice[(bytes_per_component * 1)..(bytes_per_component * 2)],
                    bit_depth,
                ),
                b: compress_to_u8(
                    &slice[(bytes_per_component * 2)..(bytes_per_component * 3)],
                    bit_depth,
                ),
                a: 1,
            }),
            ColorType::Indexed => None,
            ColorType::GrayscaleAlpha => {
                let g = compress_to_u8(&slice[0..bytes_per_component], bit_depth);
                Some(Self {
                    r: g,
                    g: g,
                    b: g,
                    a: compress_to_u8(
                        &slice[bytes_per_component..(bytes_per_component + 1)],
                        bit_depth,
                    ),
                })
            }
            ColorType::Rgba => Some(Self {
                r: compress_to_u8(&slice[0..bytes_per_component], bit_depth),
                g: compress_to_u8(
                    &slice[(bytes_per_component * 1)..(bytes_per_component * 2)],
                    bit_depth,
                ),
                b: compress_to_u8(
                    &slice[(bytes_per_component * 2)..(bytes_per_component * 3)],
                    bit_depth,
                ),
                a: compress_to_u8(
                    &slice[(bytes_per_component * 3)..(bytes_per_component * 4)],
                    bit_depth,
                ),
            }),
        }
    }

    fn brightness(&self) -> f32 {
        ((self.r as f32) * 0.2126 + (self.g as f32) * 0.7152 + (self.b as f32) * 0.0722)
            * (self.a as f32)
    }
}

fn compress_to_u8(slice: &[u8], bit_depth: BitDepth) -> u8 {
    match bit_depth {
        BitDepth::One => slice[0],
        BitDepth::Two => slice[0],
        BitDepth::Four => slice[0],
        BitDepth::Eight => slice[0],
        BitDepth::Sixteen => {
            let (int_bytes, _) = slice.split_at(std::mem::size_of::<u128>());
            (u128::from_be_bytes(int_bytes.try_into().unwrap()) / (1 << (128 - 8))) as u8
        }
    }
}

fn assume_u8(slice: &[u8], bit_depth: BitDepth) -> u8 {
    match bit_depth {
        BitDepth::One => slice[0],
        BitDepth::Two => slice[0],
        BitDepth::Four => slice[0],
        BitDepth::Eight => slice[0],
        BitDepth::Sixteen => {
            let (int_bytes, _) = slice.split_at(std::mem::size_of::<u16>());
            (u16::from_be_bytes(int_bytes.try_into().unwrap())) as u8
        }
    }
}

fn bit_depth_to_bytes(bit_depth: BitDepth) -> usize {
    match bit_depth {
        BitDepth::One => 1,
        BitDepth::Two => 1,
        BitDepth::Four => 1,
        BitDepth::Eight => 1,
        BitDepth::Sixteen => 2,
    }
}
