//#![allow(unused)]

pub struct BitReader<'a> {
    to_read: &'a [u8],
    pos: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(to_read: &'a [u8]) -> Self {
        Self { to_read, pos: 0 }
    }

    pub fn read_bit(&mut self) -> Option<bool> {
        let off = self.pos / 8;
        if off >= self.to_read.len() {
            return None;
        }
        let read = self.to_read[off] & (1 << (self.pos % 8)) != 0;
        self.pos += 1;

        return Some(read);
    }

    pub fn read_at(&mut self, bit_pos: usize) -> Option<bool> {
        let off = bit_pos / 8;
        if off >= self.to_read.len() {
            return None;
        }

        let read = self.to_read[off] & (1 << (bit_pos % 8)) != 0;
        self.pos += 1;

        return Some(read);
    }
}

pub trait BitWriter {
    fn write_bit(&mut self, bit: u8);
    fn write_bit_at(&mut self, bit: u8, bit_pos: usize);
    fn write(&mut self, val: u32, len: usize);
    fn read_at(&self, bit_pos: usize) -> Option<bool>;
    fn swap(&mut self, bit_pos_1: usize, bit_pos_2: usize);
}


#[derive(Debug)]
pub struct BitVecWriter<'a> {
    to_write: &'a mut Vec<u8>,
    pos: usize,
}

impl<'a> BitVecWriter<'a> {
    pub fn new(to_write: &'a mut Vec<u8>) -> Self {
        to_write.clear();
        Self { to_write, pos: 0 }
    }
}

impl<'a> BitWriter for BitVecWriter<'a> {
    fn write_bit(&mut self, bit: u8) {
        assert!(bit <= 2);
        let off = self.pos / 8;
        if off >= self.to_write.len() {
            self.to_write.push(bit);
        } else {
            self.to_write[off] |= bit << (self.pos % 8);
        }

        self.pos += 1;
    }

    fn write_bit_at(&mut self, bit: u8, bit_pos: usize) {
        assert!(bit <= 2);
        let off = bit_pos / 8;
        if off >= self.to_write.len() {
            while off > self.to_write.len() {
                self.to_write.push(0);
            }
            self.to_write.push(bit);
        } else {
            self.to_write[off] |= bit << (bit_pos % 8);
        }

        self.pos += 1;
    }

    fn write(&mut self, val: u32, len: usize) {
        for i in (0..len).rev() {
            self.write_bit(((val & (1 << i)) >> i) as u8);
        }
    }

    fn read_at(&self, bit_pos: usize) -> Option<bool> {
        let off = bit_pos / 8;
        if off >= self.to_write.len() {
            return None;
        }

        let read = self.to_write[off] & (1 << (bit_pos % 8)) != 0;

        return Some(read);
    }

    fn swap(&mut self, bit_pos_1: usize, bit_pos_2: usize) {
        let b1 = self.read_at(bit_pos_1).unwrap();
        let b2 = self.read_at(bit_pos_2).unwrap();

        self.write_bit_at(b1 as u8, bit_pos_2);
        self.write_bit_at(b2 as u8, bit_pos_1);
    }
}
