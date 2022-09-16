#![no_std]

use core::convert::TryFrom;

#[repr(u8)]
pub enum BitsPerPixel {
    One = 1,
    Two = 2,
}

impl BitsPerPixel {
    pub fn get_num(&self) -> u32 {
        match self {
            BitsPerPixel::One => 1,
            BitsPerPixel::Two => 2,
        }
    }

    pub fn get_flags(&self) -> u8 {
        match self {
            BitsPerPixel::One => 0,
            BitsPerPixel::Two => 1,
        }
    }

    pub fn try_from_flags(flags: u8) -> Result<Self, &'static str> {
        match flags {
            0 => Ok(BitsPerPixel::One),
            1 => Ok(BitsPerPixel::Two),
            _ => Err("Flags must be 0 or 1"),
        }
    }
}

#[repr(u8)]
pub enum CompType {
    Uncompressed,
    Pk,
}

impl TryFrom<u8> for CompType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CompType::Uncompressed),
            1 => Ok(CompType::Pk),
            _ => Err("Invalid compression type"),
        }
    }
}
