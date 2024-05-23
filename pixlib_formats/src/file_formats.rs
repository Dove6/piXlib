use codepage_strings::{Coding, ConvertError};
use lazy_static::lazy_static;

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
