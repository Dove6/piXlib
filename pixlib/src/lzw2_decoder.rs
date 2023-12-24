use core::panic;

pub trait IntoConst {
    fn into_const<const L: usize>(&self) -> [u8; L];
}

impl IntoConst for [u8] {
    fn into_const<const L: usize>(&self) -> [u8; L] {
        self.try_into().unwrap()
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum DecompressionError {
    NotEnoughBytes {
        actual_length: usize,
        required_length: Option<usize>,
    },
    UnknownCodeword,
}

impl<'a> CodewordIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        CodewordIterator { data, index: 0 }
    }

    fn try_increment_index(&mut self) -> bool {
        if self.index + 1 >= self.data.len() {
            return false;
        }
        self.index += 1;
        true
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
}

impl<'a> Iterator for CodewordIterator<'a> {
    type Item = Result<Codeword<'a>, DecompressionError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.len() {
            return None;
        }
        if self.index == 0 {
            let initial_literal = try_parse_initial_literal(self.data);
            if let Some(_) = initial_literal {
                return initial_literal;
            }
        }

        let original_index = self.index;
        // print!("{original_index}");
        match get_high_nibble(self.current_byte()) {
            0b0000 => {
                let length_bias: usize = 3;

                let mut length = (self.current_byte() & 0b1111) as usize;
                if length == 0 {
                    length += 15;
                    if !self.try_increment_index() {
                        return Some(Err(DecompressionError::NotEnoughBytes {
                            actual_length: 1,
                            required_length: None,
                        }));
                    }
                    while self.current_byte() == 0 {
                        length += 255;
                        if !self.try_increment_index() {
                            return Some(Err(DecompressionError::NotEnoughBytes {
                                actual_length: self.index - original_index,
                                required_length: None,
                            }));
                        }
                    }
                    length += self.current_byte() as usize;
                }

                let length = length + length_bias;

                if !self.try_increase_index(length) {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: self.data.len() - self.index,
                        required_length: Some(length),
                    }));
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

                let distance = (((self.current_byte() as u16) & 0b00001000) << 11) as usize;
                let mut length = (self.current_byte() & 0b111) as usize;
                if length == 0 {
                    length += 7;
                    if !self.try_increment_index() {
                        return Some(Err(DecompressionError::NotEnoughBytes {
                            actual_length: 1,
                            required_length: None,
                        }));
                    }
                    while self.current_byte() == 0 {
                        length += 255;
                        if !self.try_increment_index() {
                            return Some(Err(DecompressionError::NotEnoughBytes {
                                actual_length: self.index - original_index,
                                required_length: None,
                            }));
                        }
                    }
                    length += self.current_byte() as usize;
                }
                if !self.try_increment_index() {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: 0,
                        required_length: Some(2),
                    }));
                }
                let distance = distance + (self.current_byte() >> 2) as usize;
                let following_literals_length = (self.current_byte() & 0b00000011) as usize;
                if !self.try_increment_index() {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: 1,
                        required_length: Some(2),
                    }));
                }
                let distance = distance + ((self.current_byte() as u16) << 6) as usize;

                if length == 1 && distance == 0 && following_literals_length == 0 {
                    self.index += 1;
                    Some(Ok(Codeword::Terminator))
                } else {
                    let length = length + length_bias;
                    let distance = distance + distance_bias;

                    if !self.try_increase_index(following_literals_length) {
                        return Some(Err(DecompressionError::NotEnoughBytes {
                            actual_length: self.data.len() - self.index,
                            required_length: Some(following_literals_length),
                        }));
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

                let mut length = (self.current_byte() & 0b11111) as usize;
                if length == 0 {
                    length += 31;
                    if !self.try_increment_index() {
                        return Some(Err(DecompressionError::NotEnoughBytes {
                            actual_length: 1,
                            required_length: None,
                        }));
                    }
                    while self.current_byte() == 0 {
                        length += 255;
                        if !self.try_increment_index() {
                            return Some(Err(DecompressionError::NotEnoughBytes {
                                actual_length: self.index - original_index,
                                required_length: None,
                            }));
                        }
                    }
                    length += self.current_byte() as usize;
                }
                if !self.try_increment_index() {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: 0,
                        required_length: Some(2),
                    }));
                }
                let distance = ((self.current_byte() & 0b11111100) >> 2) as usize;
                let following_literals_length = (self.current_byte() & 0b00000011) as usize;
                if !self.try_increment_index() {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: 1,
                        required_length: Some(2),
                    }));
                }
                let distance = distance + ((self.current_byte() as u16) << 6) as usize;

                let length = length + length_bias;
                let distance = distance + distance_bias;

                if !self.try_increase_index(following_literals_length) {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: self.data.len() - self.index,
                        required_length: Some(following_literals_length),
                    }));
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

                let length = ((self.current_byte() & 0b11100000) >> 5) as usize;
                let distance = ((self.current_byte() & 0b00011100) >> 2) as usize;
                let following_literals_length = (self.current_byte() & 0b00000011) as usize;
                if !self.try_increment_index() {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: 1,
                        required_length: Some(2),
                    }));
                }
                let distance = distance + ((self.current_byte() as u16) << 3) as usize;

                let length = length + length_bias;
                let distance = distance + distance_bias;

                if !self.try_increase_index(following_literals_length) {
                    return Some(Err(DecompressionError::NotEnoughBytes {
                        actual_length: self.data.len() - self.index,
                        required_length: Some(following_literals_length),
                    }));
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
            _ => Some(Err(DecompressionError::UnknownCodeword)),
        }
    }
}

fn try_parse_initial_literal(
    compressed_data: &[u8],
) -> Option<Result<Codeword, DecompressionError>> {
    if get_high_nibble(compressed_data[0]) == 0b0000 {
        return None;
    }
    let literal_length = compressed_data[0] as usize - 17;
    if literal_length >= compressed_data.len() {
        return Some(Err(DecompressionError::NotEnoughBytes {
            actual_length: compressed_data.len() - 1,
            required_length: Some(literal_length),
        }));
    }
    Some(Ok(Codeword::Literals {
        detailed_type: LiteralsType::Initial,
        literals: &compressed_data[1..=literal_length],
    }))
}

pub fn decode_lzw2(data: &[u8]) -> Vec<u8> {
    let decompressed_size = u32::from_le_bytes(data[..4].into_const()) as usize;
    let compressed_size = u32::from_le_bytes(data[4..8].into_const()) as usize;
    assert_eq!(compressed_size + 8, data.len());

    let compressed_data = &data[8..];
    let mut decompressed_data = vec![0; decompressed_size];
    let mut decompressed_index = 0;

    let mut terminator_encountered = false;

    for codeword in CodewordIterator::new(compressed_data) {
        // if let Ok(Codeword::Literals { detailed_type, literals }) = codeword { println!(", {}, ll {}", match detailed_type {
        //     LiteralsType::Initial => "0001",
        //     LiteralsType::Regular => "0000",
        // }, literals.len()); }
        // if let Ok(Codeword::Reference { detailed_type, length, distance, following_literals }) = codeword { println!(", {}, l {}, d {}, ll {}", match detailed_type {
        //     ReferenceType::FarDistance => "0001",
        //     ReferenceType::Medium => "0010",
        //     ReferenceType::ShortLength => "0100",
        // }, length, distance, following_literals.len()); }
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
                detailed_type,
                length,
                distance,
                following_literals,
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
                    decompressed_data[decompressed_index] = decompressed_data[decompressed_index - distance];
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
