use codepage_strings::{Coding, ConvertError};

pub mod ann;
pub mod arr;
pub mod img;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageData {
    pub color: Vec<u8>,
    pub alpha: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum ColorFormat {
    Rgb565,
    Rgb555,
}

impl ColorFormat {
    pub fn new(bit_depth: u32) -> Self {
        match bit_depth {
            16 => Self::Rgb565,
            15 => Self::Rgb555,
            _ => panic!("Bit depth {} is not supported.", bit_depth),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum CompressionType {
    None,
    Rle,
    Lzw2,
    RleInLzw2,
    Jpeg,
}

fn from_cp1250(mut v: &[u8], null_terminated: bool) -> Result<String, ConvertError> {
    if null_terminated {
        v = v.split(|c| *c == 0).next().unwrap();
    }
    Coding::new(1250).unwrap().decode(v).map(|v| v.into_owned())
}
