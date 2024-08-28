use core::panic;

use super::DecompressionError;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum Codeword<'a> {
    Literals {
        detailed_type: LiteralsType,
        literals: &'a [u8],
    },
    Reference {
        detailed_type: ReferenceType,
        length: usize,
        distance: usize,
        following_literals: &'a [u8],
    },
    Terminator,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum LiteralsType {
    Initial,
    Regular,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum ReferenceType {
    FarDistance, // 0001 MSB
    Medium,      // 001 MSB
    ShortLength, // 01 or 1 MSB
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
struct CodewordIterator<'a> {
    data: &'a [u8],
    index: usize,
}

impl<'a> CodewordIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        CodewordIterator { data, index: 0 }
    }

    fn try_increment_index(&mut self) -> bool {
        self.try_increase_index(1)
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
        if self.index == 0 && get_high_nibble(self.current_byte()) != 0b0000 {
            let length = Into::<usize>::into(self.current_byte()) - 17;
            if !self.try_increment_index() {
                return Some(Err(self.create_not_enough_bytes_error(1, None)));
            }
            let literals = &self.data[self.index..(self.index + length)];
            if !self.try_increase_index(length) {
                return Some(Err(self.create_not_enough_bytes_error(
                    self.data.len() - self.index,
                    Some(length),
                )));
            }
            return Some(Ok(Codeword::Literals {
                detailed_type: LiteralsType::Initial,
                literals,
            }));
        }

        let original_index = self.index;
        match get_high_nibble(self.current_byte()) {
            0b0000 => {
                let length_bias: usize = 3;

                let mut length: usize = (self.current_byte() & 0b1111).into();
                if length == 0 {
                    length += 15;
                    if !self.try_increment_index() {
                        return Some(Err(self.create_not_enough_bytes_error(1, None)));
                    }
                    while self.current_byte() == 0 {
                        length += 255;
                        if !self.try_increment_index() {
                            return Some(Err(self.create_not_enough_bytes_error(
                                self.index - original_index,
                                None,
                            )));
                        }
                    }
                    length += Into::<usize>::into(self.current_byte());
                }

                let length = length + length_bias;

                if !self.try_increase_index(length) {
                    return Some(Err(self.create_not_enough_bytes_error(
                        self.data.len() - self.index,
                        Some(length),
                    )));
                }
                self.index += 1;
                let literals = &self.data[self.index - length..self.index];

                Some(Ok(Codeword::Literals {
                    detailed_type: LiteralsType::Regular,
                    literals,
                }))
            }
            0b0001 => {
                let length_bias: usize = 2;
                let distance_bias: usize = 16384;

                let distance = Into::<usize>::into(self.current_byte() & 0b00001000) << 11;
                let mut length: usize = (self.current_byte() & 0b111).into();
                if length == 0 {
                    length += 7;
                    if !self.try_increment_index() {
                        return Some(Err(self.create_not_enough_bytes_error(1, None)));
                    }
                    while self.current_byte() == 0 {
                        length += 255;
                        if !self.try_increment_index() {
                            return Some(Err(self.create_not_enough_bytes_error(
                                self.index - original_index,
                                None,
                            )));
                        }
                    }
                    length += Into::<usize>::into(self.current_byte());
                }
                if !self.try_increment_index() {
                    return Some(Err(self.create_not_enough_bytes_error(0, Some(2))));
                }
                let distance: usize = distance + (Into::<usize>::into(self.current_byte()) >> 2);
                let following_literals_length: usize = (self.current_byte() & 0b00000011).into();
                if !self.try_increment_index() {
                    return Some(Err(self.create_not_enough_bytes_error(1, Some(2))));
                }
                let distance: usize = distance + (Into::<usize>::into(self.current_byte()) << 6);

                if length == 1 && distance == 0 && following_literals_length == 0 {
                    self.index += 1;
                    Some(Ok(Codeword::Terminator))
                } else {
                    let length = length + length_bias;
                    let distance = distance + distance_bias;

                    if !self.try_increase_index(following_literals_length) {
                        return Some(Err(self.create_not_enough_bytes_error(
                            self.data.len() - self.index,
                            Some(following_literals_length),
                        )));
                    }
                    self.index += 1;
                    let following_literals =
                        &self.data[self.index - following_literals_length..self.index];

                    Some(Ok(Codeword::Reference {
                        detailed_type: ReferenceType::FarDistance,
                        length,
                        distance,
                        following_literals,
                    }))
                }
            }
            0b0010..=0b0011 => {
                let length_bias: usize = 2;
                let distance_bias: usize = 1;

                let mut length: usize = (self.current_byte() & 0b11111).into();
                if length == 0 {
                    length += 31;
                    if !self.try_increment_index() {
                        return Some(Err(self.create_not_enough_bytes_error(1, None)));
                    }
                    while self.current_byte() == 0 {
                        length += 255;
                        if !self.try_increment_index() {
                            return Some(Err(self.create_not_enough_bytes_error(
                                self.index - original_index,
                                None,
                            )));
                        }
                    }
                    length += Into::<usize>::into(self.current_byte());
                }
                if !self.try_increment_index() {
                    return Some(Err(self.create_not_enough_bytes_error(0, Some(2))));
                }
                let distance = Into::<usize>::into(self.current_byte() & 0b11111100) >> 2;
                let following_literals_length: usize = (self.current_byte() & 0b00000011).into();
                if !self.try_increment_index() {
                    return Some(Err(self.create_not_enough_bytes_error(1, Some(2))));
                }
                let distance = distance + (Into::<usize>::into(self.current_byte()) << 6);

                let length = length + length_bias;
                let distance = distance + distance_bias;

                if !self.try_increase_index(following_literals_length) {
                    return Some(Err(self.create_not_enough_bytes_error(
                        self.data.len() - self.index,
                        Some(following_literals_length),
                    )));
                }
                self.index += 1;
                let following_literals =
                    &self.data[self.index - following_literals_length..self.index];

                Some(Ok(Codeword::Reference {
                    detailed_type: ReferenceType::Medium,
                    length,
                    distance,
                    following_literals,
                }))
            }
            0b0100..=0b1111 => {
                let length_bias: usize = 1;
                let distance_bias: usize = 1;

                let length = Into::<usize>::into(self.current_byte() & 0b11100000) >> 5;
                let distance = Into::<usize>::into(self.current_byte() & 0b00011100) >> 2;
                let following_literals_length: usize = (self.current_byte() & 0b00000011).into();
                if !self.try_increment_index() {
                    return Some(Err(self.create_not_enough_bytes_error(1, Some(2))));
                }
                let distance = distance + (Into::<usize>::into(self.current_byte()) << 3);

                let length = length + length_bias;
                let distance = distance + distance_bias;

                if !self.try_increase_index(following_literals_length) {
                    return Some(Err(self.create_not_enough_bytes_error(
                        self.data.len() - self.index,
                        Some(following_literals_length),
                    )));
                }
                self.index += 1;
                let following_literals =
                    &self.data[self.index - following_literals_length..self.index];

                Some(Ok(Codeword::Reference {
                    detailed_type: ReferenceType::ShortLength,
                    length,
                    distance,
                    following_literals,
                }))
            }
            _ => Some(Err(DecompressionError::new(
                self.index,
                super::DecompressionErrorKind::UnknownCodeword,
            ))),
        }
    }
}

pub fn decode_lzw2(data: &[u8]) -> Vec<u8> {
    let decompressed_size: usize = u32::from_le_bytes(data[..4].try_into().unwrap())
        .try_into()
        .unwrap();
    let compressed_size: usize = u32::from_le_bytes(data[4..8].try_into().unwrap())
        .try_into()
        .unwrap();
    assert_eq!(compressed_size + 8, data.len());

    let compressed_data = &data[8..];
    let mut decompressed_data = vec![0; decompressed_size];
    let mut decompressed_index = 0;

    let mut terminator_encountered = false;

    for codeword in CodewordIterator::new(compressed_data) {
        match codeword {
            Ok(_) if terminator_encountered => panic!("Codewords after terminator"),
            Ok(Codeword::Literals { literals, .. }) => {
                if decompressed_index + literals.len() > decompressed_data.len() {
                    panic!(
                        "Decompressed data too large: {} / {}",
                        decompressed_index + literals.len(),
                        decompressed_data.len()
                    );
                }
                decompressed_data[decompressed_index..decompressed_index + literals.len()]
                    .copy_from_slice(literals);
                decompressed_index += literals.len();
            }
            Ok(Codeword::Reference {
                length,
                distance,
                following_literals,
                ..
            }) => {
                if decompressed_index + length + following_literals.len() > decompressed_data.len()
                {
                    panic!(
                        "Decompressed data too large: {} / {}",
                        decompressed_index + length + following_literals.len(),
                        decompressed_data.len()
                    );
                }
                for _ in 0..length {
                    decompressed_data[decompressed_index] =
                        decompressed_data[decompressed_index - distance];
                    decompressed_index += 1;
                }
                decompressed_data
                    [decompressed_index..decompressed_index + following_literals.len()]
                    .copy_from_slice(following_literals);
                decompressed_index += following_literals.len();
            }
            Ok(Codeword::Terminator) => terminator_encountered = true,
            Err(error) => panic!("Decompression error: {:?}", error),
        }
    }

    assert!(terminator_encountered);
    assert_eq!(decompressed_index, decompressed_size);

    decompressed_data
}

fn get_high_nibble(byte: u8) -> u8 {
    (byte & 0b11110000) >> 4
}

#[cfg(test)]
mod test_lzw2_decoder {
    use super::*;

    #[test]
    fn initial_literals_should_decompress_successfully() {
        assert_eq!(
            decode_lzw2(&[
                0x02, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x11, 0x00, 0x00
            ]),
            &[0x00, 0x00]
        );
    }
}
