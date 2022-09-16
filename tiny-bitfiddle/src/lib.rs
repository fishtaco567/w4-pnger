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

    pub fn peek_bit(&self) -> Option<bool> {
        let off = self.pos / 8;
        if off >= self.to_read.len() {
            return None;
        }
        let read = self.to_read[off] & (1 << (self.pos % 8)) != 0;

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
        debug_assert!(bit <= 2);
        let off = self.pos / 8;
        if off >= self.to_write.len() {
            self.to_write.push(bit);
        } else {
            self.to_write[off] |= bit << (self.pos % 8);
        }

        self.pos += 1;
    }

    fn write_bit_at(&mut self, bit: u8, bit_pos: usize) {
        debug_assert!(bit <= 2);
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

pub struct BitSliceWriter<'a> {
    bit_slice: &'a mut [u8],
    pos: usize,
}

impl<'a> BitSliceWriter<'a> {
    pub fn new(bit_slice: &'a mut [u8]) -> Self {
        Self { bit_slice, pos: 0 }
    }

    pub fn rotate_right(&mut self, start: usize, end: usize, amt: usize) {
        self.reverse(start, end);
        self.reverse(start, start + amt);
        self.reverse(start + amt, end);
    }

    fn reverse(&mut self, start: usize, end: usize) {
        let mut i = start;
        let mut j = end - 1;
        while i < j {
            self.swap(i, j);
            i += 1;
            j -= 1;
        }
    }

    pub fn get_end(&self) -> usize {
        self.pos
    }
}

impl<'a> BitWriter for BitSliceWriter<'a> {
    fn write_bit(&mut self, bit: u8) {
        debug_assert!(bit <= 2);
        debug_assert!(self.pos / 8 < self.bit_slice.len());

        let off = self.pos / 8;
        let mask = !(1 << self.pos % 8);
        self.bit_slice[off] = (self.bit_slice[off] & mask) | (bit << (self.pos % 8));

        self.pos += 1;
    }

    fn write_bit_at(&mut self, bit: u8, bit_pos: usize) {
        debug_assert!(bit <= 2);
        debug_assert!(bit_pos / 8 < self.bit_slice.len());

        let off = bit_pos / 8;
        let mask = !(1 << bit_pos % 8);
        self.bit_slice[off] = (self.bit_slice[off] & mask) | (bit << (bit_pos % 8));

        self.pos += 1;
    }

    fn write(&mut self, val: u32, len: usize) {
        debug_assert!((self.pos + len) / 8 < self.bit_slice.len());

        for i in (0..len).rev() {
            self.write_bit(((val & (1 << i)) >> i) as u8);
        }
    }

    fn read_at(&self, bit_pos: usize) -> Option<bool> {
        debug_assert!(bit_pos / 8 < self.bit_slice.len());

        let off = bit_pos / 8;
        if off >= self.bit_slice.len() {
            return None;
        }

        let read = self.bit_slice[off] & (1 << (bit_pos % 8)) != 0;

        return Some(read);
    }

    fn swap(&mut self, bit_pos_1: usize, bit_pos_2: usize) {
        debug_assert!(bit_pos_1 / 8 < self.bit_slice.len());
        debug_assert!(bit_pos_2 / 8 < self.bit_slice.len());

        let b1 = self.read_at(bit_pos_1).unwrap();
        let b2 = self.read_at(bit_pos_2).unwrap();

        self.write_bit_at(b1 as u8, bit_pos_2);
        self.write_bit_at(b2 as u8, bit_pos_1);
    }
}
