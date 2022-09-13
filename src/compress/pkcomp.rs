use std::{fmt::Debug};

use super::{
    bitfiddle::{BitReader, BitWriter},
    delta_encode, delta_encode_by_jump, split_bitplanes, xor_bitplanes,
    CompressionResult, Compressor,
};

use anyhow::Result;

pub struct PkComp;

impl Compressor for PkComp {
    fn compress(&mut self, png: &Vec<u8>) -> Result<CompressionResult> {
        let mut best = None;
        let mut best_score = usize::MAX;

        let mut best_split = false;
        let mut best_xor = false;
        let mut best_seq = 0;
        let mut best_jump = 0;
        let mut best_size = 0;

        for split in 0..=1 {
            for xor in 0..=1 {
                for seq_delta in 0..=4 {
                    for jump_delta in 0..=2 {
                        for jump_size in (4..=32).step_by(2) {
                            let out_vec = compress_for(
                                png,
                                split != 0,
                                xor != 0,
                                seq_delta,
                                jump_delta,
                                jump_size,
                            );
                            if out_vec.len() < best_score {
                                best_score = out_vec.len();
                                best = Some(out_vec);

                                best_split = split != 0;
                                best_xor = xor != 0;
                                best_seq = seq_delta;
                                best_jump = jump_delta;
                                best_size = jump_size;
                            }
                        }
                    }
                }
            }
        }

        let mut out_header = vec![0, best_size as u8];
        out_header[0] |= if best_split { 1 << 0 } else { 0 };
        out_header[0] |= if best_xor { 1 << 1 } else { 0 };
        out_header[0] |= (best_seq as u8) << 2;
        out_header[0] |= (best_jump as u8) << 6;
        let out_content = best.unwrap();

        let len = out_content.len() + out_header.len();

        Ok(CompressionResult {
            content_bytes: out_content,
            header_bytes: out_header,
            total_size: len,
            readable_compression_name: "PnTree".to_owned(),
        })
    }
}

fn compress_for(
    png: &Vec<u8>,
    do_split_bitplanes: bool,
    do_xor_bitplanes: bool,
    seq_delta_encode: usize,
    jump_delta_encode: usize,
    jump_delta_encode_size: usize,
) -> Vec<u8> {
    let mut cloned = png.clone();

    let mut out_vec = Vec::new();

    if do_split_bitplanes {
        let mut bitplane_1 = Vec::with_capacity(cloned.len() / 2);
        let mut bitplane_2 = Vec::with_capacity(cloned.len() / 2);
        split_bitplanes(&cloned, &mut bitplane_1, &mut bitplane_2);
        if do_xor_bitplanes {
            xor_bitplanes(&bitplane_2, &mut bitplane_1);
        }

        let mut out_bp1 = compress_bitplane(
            &mut bitplane_1,
            seq_delta_encode,
            jump_delta_encode,
            jump_delta_encode_size,
        );
        let mut out_bp2 = compress_bitplane(
            &mut bitplane_2,
            seq_delta_encode,
            jump_delta_encode,
            jump_delta_encode_size,
        );

        out_bp1.append(&mut out_bp2);
        out_vec.append(&mut out_bp1);
    } else {
        out_vec.append(&mut compress_bitplane(
            &mut cloned,
            seq_delta_encode,
            jump_delta_encode,
            jump_delta_encode_size,
        ));
    }

    out_vec
}

fn compress_bitplane(
    bytes: &mut Vec<u8>,
    seq_delta_encode: usize,
    jump_delta_encode: usize,
    jump_delta_encode_size: usize,
) -> Vec<u8> {
    let mut double_buffer = Vec::with_capacity(bytes.len());
    for _ in 0..seq_delta_encode {
        delta_encode(&bytes, &mut double_buffer);
        std::mem::swap(bytes, &mut double_buffer);
    }

    for _ in 0..jump_delta_encode {
        delta_encode_by_jump(&bytes, &mut double_buffer, jump_delta_encode_size);
        std::mem::swap(bytes, &mut double_buffer);
    }

    let mut reader = BitReader::new(&bytes);

    let mut out_vec: Vec<u8> = Vec::new();
    let mut writer = BitWriter::new(&mut out_vec);

    let b1 = reader.read_bit().unwrap();
    let b2 = reader.read_bit().unwrap();
    let mut state = match (b1, b2) {
        (true, true) => State::Root(b1, b2, 1),
        (true, false) => State::Root(b1, b2, 1),
        (false, true) => State::Root(b1, b2, 1),
        (false, false) => State::Zeroes(1),
    };

    loop {
        let b1 = match reader.read_bit() {
            Some(b) => b,
            None => {
                write(state, &mut writer);
                break;
            } //Found end-of-stream
        };

        let b2 = match reader.read_bit() {
            Some(b) => b,
            None => false, //Odd number of pairs, fill in 0 for the last bit
        };

        state = match state {
            State::Zeroes(n) => match (b1, b2) {
                (false, false) => State::Zeroes(n + 1),
                (true, false) | (false, true) | (true, true) => {
                    write(state, &mut writer);
                    State::Root(b1, b2, 1)
                }
            },
            State::Root(lb1, lb2, i) => match (b1, b2) {
                (false, false) => {
                    if i == 1 && (lb1, lb2) != (true, true) {
                        writer.write_bit(0);
                        writer.write_bit((b1 == true) as u8);
                    } else {
                        write(state, &mut writer);
                        writer.write_bit(0);
                        writer.write_bit(0);
                    }
                    State::Zeroes(1)
                }
                (true, true) | (true, false) | (false, true) => {
                    if i == 1 {
                        writer.write_bit(1);
                    }
                    write(state, &mut writer);
                    State::Root(b1, b2, i + 1)
                }
            },
        };
    }

    out_vec
}

fn write(prev_state: State, writer: &mut BitWriter) {
    match prev_state {
        State::Zeroes(n) => {
            let n = n + 1;
            let hb = highest_bit(n);
            let f = 1 << (hb - 1);
            let v = n & !f;
            let l = f - 2;

            writer.write(l as u32, hb - 1);
            writer.write(v as u32, hb - 1);
        }
        State::Root(b1, b2, _) => {
            _ = writer.write_bit(b1 as u8);
            _ = writer.write_bit(b2 as u8);
        }
    }
}

fn highest_bit(mut n: usize) -> usize {
    let mut bit: usize = 0;
    while n > 0 {
        n = n >> 1;
        bit += 1;
    }

    bit
}

#[derive(Debug, Clone, Copy)]
enum State {
    Zeroes(usize),
    Root(bool, bool, usize),
}
