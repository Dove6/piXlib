mod ann;
mod arr;
mod img;

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
