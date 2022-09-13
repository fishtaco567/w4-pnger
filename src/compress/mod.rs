mod bitfiddle;
pub mod pkcomp;

use anyhow::Result;

use self::bitfiddle::{BitReader, BitWriter};

pub trait Compressor {
    fn compress(&mut self, png: &Vec<u8>) -> Result<CompressionResult>;
}

pub struct CompressionResult {
    pub content_bytes: Vec<u8>,
    pub header_bytes: Vec<u8>,
    pub total_size: usize,
    pub readable_compression_name: String,
}

#[repr(u8)]
pub enum CompType {
    Uncompressed,
    Pk,
}

fn delta_encode(in_bytes: &Vec<u8>, out_bytes: &mut Vec<u8>) {
    let mut writer = BitWriter::new(out_bytes);

    let mut reader = BitReader::new(in_bytes);

    let mut last = false;

    loop {
        let b1 = match reader.read_bit() {
            Some(b) => b,
            None => break,
        };

        if b1 == last {
            writer.write_bit(0);
        } else {
            writer.write_bit(1);
        }

        last = b1;
    }
}

fn delta_decode(in_bytes: &Vec<u8>, out_bytes: &mut Vec<u8>) {
    let mut writer = BitWriter::new(out_bytes);

    let mut reader = BitReader::new(in_bytes);

    let mut cur = 0;

    loop {
        let b1 = match reader.read_bit() {
            Some(b) => b,
            None => break,
        };

        if b1 {
            cur = 1 - cur;
        }
        _ = writer.write_bit(cur);
    }
}

pub fn delta_encode_by_jump(in_bytes: &Vec<u8>, out_bytes: &mut Vec<u8>, jump: usize) {
    let mut writer = BitWriter::new(out_bytes);

    let mut reader = BitReader::new(in_bytes);

    for _ in 0..jump {
        writer.write_bit(reader.read_bit().unwrap() as u8);
    }

    for i in jump..(in_bytes.len() * 8) {
        let b1 = reader.read_at(i - jump).unwrap();

        let b2 = reader.read_at(i).unwrap();

        if b1 == b2 {
            writer.write_bit(0);
        } else {
            writer.write_bit(1);
        }
    }
}

fn split_bitplanes(in_bytes: &Vec<u8>, out_left: &mut Vec<u8>, out_right: &mut Vec<u8>) {
    let mut writer_1 = BitWriter::new(out_left);
    let mut writer_2 = BitWriter::new(out_right);

    let mut reader = BitReader::new(&in_bytes);

    let mut write_1 = true;

    loop {
        let b1 = match reader.read_bit() {
            Some(b) => b,
            None => break,
        } as u8;

        if write_1 {
            writer_1.write_bit(b1);
        } else {
            writer_2.write_bit(b1);
        }

        write_1 = !write_1;
    }
}

fn xor_bitplanes(bp1: &Vec<u8>, bp2: &mut Vec<u8>) {
    for (b1, b2) in bp1.iter().zip(bp2.iter_mut()) {
        *b2 = *b1 ^ *b2;
    }
}
