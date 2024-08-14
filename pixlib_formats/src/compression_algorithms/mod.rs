use std::fmt::Display;

pub mod lzw2;
pub mod rle;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct DecompressionError {
    pub position: usize,
    pub kind: DecompressionErrorKind,
}

impl DecompressionError {
    pub fn new(position: usize, kind: DecompressionErrorKind) -> Self {
        Self { position, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum DecompressionErrorKind {
    NotEnoughBytes {
        actual_length: usize,
        required_length: Option<usize>,
    },
    UnknownCodeword,
}

impl Display for DecompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position = self.position;
        match self.kind {
            DecompressionErrorKind::NotEnoughBytes {
                actual_length,
                required_length: Some(required_length),
            } => write!(
                f,
                "Not enough bytes (required: {}, actual: {}) at position {}!",
                required_length, actual_length, position
            ),
            DecompressionErrorKind::NotEnoughBytes {
                actual_length,
                required_length: None,
            } => write!(
                f,
                "Not enough bytes ({}) at position {}!",
                actual_length, position
            ),
            DecompressionErrorKind::UnknownCodeword => {
                write!(f, "Unrecognized codeword at position {}!", position)
            }
        }
    }
}
