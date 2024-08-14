use std::sync::Arc;

use codepage_strings::{Coding, ConvertError};
use lazy_static::lazy_static;

use crate::compression_algorithms::{lzw2::decode_lzw2, rle::decode_rle};

pub mod ann;
pub mod arr;
pub mod img;

lazy_static! {
    pub static ref STRING_ENCODING: Coding = Coding::new(1250).unwrap();
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageData<'a> {
    pub color: &'a [u8],
    pub alpha: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum ColorFormat {
    Rgb565,
    Rgb555,
}

impl ColorFormat {
    pub fn new(bit_depth: u32) -> Result<Self, String> {
        match bit_depth {
            16 => Ok(Self::Rgb565),
            15 => Ok(Self::Rgb555),
            _ => Err(format!("Bit depth {} is not supported.", bit_depth)),
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

impl<'a> ImageData<'a> {
    pub fn to_rgba8888(&self, format: ColorFormat, compression: CompressionType) -> Arc<[u8]> {
        let has_alpha = !self.alpha.is_empty();
        let color_data = match compression {
            CompressionType::None => self.color.to_owned(),
            CompressionType::Rle => decode_rle(self.color, 2),
            CompressionType::Lzw2 => decode_lzw2(self.color),
            CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(self.color), 2),
            _ => panic!(),
        };
        let alpha_data = match compression {
            _ if !has_alpha => vec![],
            CompressionType::None => self.alpha.to_owned(),
            CompressionType::Rle => decode_rle(self.alpha, 1),
            CompressionType::Lzw2 | CompressionType::Jpeg => decode_lzw2(self.alpha),
            CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(self.alpha), 1),
        };
        assert!(color_data.len() % 2 == 0);
        if has_alpha {
            assert!(alpha_data.len() * 2 == color_data.len());
        }
        let target_length = color_data.len() * 2;
        let mut data = vec![255; target_length];
        match format {
            ColorFormat::Rgb565 => {
                for i in 0..(color_data.len() / 2) {
                    let rgb565_l = color_data[2 * i];
                    let rgb565_h = color_data[2 * i + 1];
                    let r5: u16 = ((rgb565_h >> 3) & 0x1f).into();
                    let g6: u16 = (((rgb565_l >> 5) | (rgb565_h << 3)) & 0x3f).into();
                    let b5: u16 = (rgb565_l & 0x1f).into();
                    data[4 * i] = (r5 * 255 / 31).try_into().unwrap();
                    data[4 * i + 1] = (g6 * 255 / 63).try_into().unwrap();
                    data[4 * i + 2] = (b5 * 255 / 31).try_into().unwrap();
                    if has_alpha {
                        data[4 * i + 3] = alpha_data[i];
                    }
                }
            }
            ColorFormat::Rgb555 => {
                for i in 0..(color_data.len() / 2) {
                    let rgb555_l = color_data[2 * i];
                    let rgb555_h = color_data[2 * i + 1];
                    let r5: u16 = ((rgb555_h >> 2) & 0x1f).into();
                    let g5: u16 = (((rgb555_l >> 5) | (rgb555_h << 3)) & 0x1f).into();
                    let b5: u16 = (rgb555_l & 0x1f).into();
                    data[4 * i] = (r5 * 255 / 31).try_into().unwrap();
                    data[4 * i + 1] = (g5 * 255 / 31).try_into().unwrap();
                    data[4 * i + 2] = (b5 * 255 / 31).try_into().unwrap();
                    if has_alpha {
                        data[4 * i + 3] = alpha_data[i];
                    }
                }
            }
        }
        data.into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedStr(pub String, pub Option<Vec<u8>>);

impl DecodedStr {
    pub fn rest(&self) -> &Option<Vec<u8>> {
        &self.1
    }

    pub fn with_rest(self, rest: Option<Vec<u8>>) -> Self {
        Self(self.0, rest)
    }

    pub fn total_length(&self) -> usize {
        self.0.len() + self.1.as_ref().map(Vec::<u8>::len).unwrap_or(0)
    }

    pub fn is_totally_empty(&self) -> bool {
        self.0.is_empty() && self.1.as_ref().map(Vec::<u8>::is_empty).unwrap_or(true)
    }

    pub fn from_bytes(src: &[u8]) -> Result<Self, ConvertError> {
        STRING_ENCODING
            .decode(src)
            .map(|s| Self(s.into_owned(), None))
    }

    pub fn from_bytes_null_terminated(src: &[u8]) -> Result<Self, ConvertError> {
        let null_index = src.iter().position(|c| *c == b'\0').unwrap_or(src.len());
        let rest = if null_index < src.len() {
            Some(src[null_index..].to_owned())
        } else {
            None
        };
        Self::from_bytes(&src[..null_index]).map(|s| s.with_rest(rest))
    }
}

impl AsRef<str> for DecodedStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
