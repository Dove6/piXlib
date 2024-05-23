use core::panic;

use super::DecompressionError;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum Codeword<'a> {
    Literal {
        byte_offset: usize,
        literals: &'a [u8],
    },
    Encoded {
        byte_offset: usize,
        literals: &'a [u8],
        count: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
struct CodewordIterator<'a> {
    data: &'a [u8],
    index: usize,
    element_size: usize,
}

impl<'a> CodewordIterator<'a> {
    pub fn new(data: &'a [u8], element_size: usize) -> Self {
        CodewordIterator {
            data,
            index: 0,
            element_size,
        }
    }

    fn try_increase_index(&mut self, length: usize) -> bool {
        if self.index + length >= self.data.len() {
            return false;
        }
        self.index += length;
        true
    }

    fn current_byte(&self) -> u8 {
        self.data[self.index]
    }

    fn create_not_enough_bytes_error(
        &self,
        actual_length: usize,
        required_length: Option<usize>,
    ) -> DecompressionError {
        DecompressionError {
            position: self.index,
            kind: super::DecompressionErrorKind::NotEnoughBytes {
                actual_length,
                required_length,
            },
        }
    }
}

impl<'a> Iterator for CodewordIterator<'a> {
    type Item = Result<Codeword<'a>, DecompressionError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.len() {
            return None;
        }

        let byte_offset = self.index;
        if get_most_significant_bit(self.current_byte()) {
            let count: usize = (self.current_byte() & 0b01111111).into();

            if !self.try_increase_index(self.element_size) {
                return Some(Err(
                    self.create_not_enough_bytes_error(0, Some(self.element_size))
                ));
            }
            self.index += 1;
            let literals = &self.data[self.index - self.element_size..self.index];

            Some(Ok(Codeword::Encoded {
                byte_offset,
                literals,
                count,
            }))
        } else {
            let count: usize = (self.current_byte() & 0b01111111).into();

            if !self.try_increase_index(count * self.element_size) {
                return Some(Err(self.create_not_enough_bytes_error(
                    self.data.len() - self.index,
                    Some(count * self.element_size),
                )));
            }
            self.index += 1;
            let literals = &self.data[self.index - count * self.element_size..self.index];

            Some(Ok(Codeword::Literal {
                byte_offset,
                literals,
            }))
        }
    }
}

pub fn decode_rle(data: &[u8], element_size: usize) -> Vec<u8> {
    let compressed_data = data;
    let mut decompressed_data = Vec::<u8>::new();

    for codeword in CodewordIterator::new(compressed_data, element_size) {
        match codeword {
            Ok(Codeword::Literal { literals, .. }) => {
                decompressed_data.extend_from_slice(literals);
            }
            Ok(Codeword::Encoded {
                literals, count, ..
            }) => {
                decompressed_data.extend(literals.iter().cycle().take(count * element_size));
            }
            Err(error) => panic!("Decompression error: {:?}", error),
        }
    }

    decompressed_data
}

fn get_most_significant_bit(byte: u8) -> bool {
    (byte & 0b10000000) >> 7 == 1
}
