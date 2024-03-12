use std::io::Read;

use crate::common::{Bounds, Element, Position, PositionalIterator};

lazy_static::lazy_static! {
    pub static ref CP1250_LUT: [char; 128] = [
        0x20AC, 0x0081, 0x201A, 0x0083, 0x201E, 0x2026, 0x2020, 0x2021,
        0x0088, 0x2030, 0x0160, 0x2039, 0x015A, 0x0164, 0x017D, 0x0179,
        0x0090, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014,
        0x0098, 0x2122, 0x0161, 0x203A, 0x015B, 0x0165, 0x017E, 0x017A,
        0x00A0, 0x02C7, 0x02D8, 0x0141, 0x00A4, 0x0104, 0x00A6, 0x00A7,
        0x00A8, 0x00A9, 0x015E, 0x00AB, 0x00AC, 0x00AD, 0x00AE, 0x017B,
        0x00B0, 0x00B1, 0x02DB, 0x0142, 0x00B4, 0x00B5, 0x00B6, 0x00B7,
        0x00B8, 0x0105, 0x015F, 0x00BB, 0x013D, 0x02DD, 0x013E, 0x017C,
        0x0154, 0x00C1, 0x00C2, 0x0102, 0x00C4, 0x0139, 0x0106, 0x00C7,
        0x010C, 0x00C9, 0x0118, 0x00CB, 0x011A, 0x00CD, 0x00CE, 0x010E,
        0x0110, 0x0143, 0x0147, 0x00D3, 0x00D4, 0x0150, 0x00D6, 0x00D7,
        0x0158, 0x016E, 0x00DA, 0x0170, 0x00DC, 0x00DD, 0x0162, 0x00DF,
        0x0155, 0x00E1, 0x00E2, 0x0103, 0x00E4, 0x013A, 0x0107, 0x00E7,
        0x010D, 0x00E9, 0x0119, 0x00EB, 0x011B, 0x00ED, 0x00EE, 0x010F,
        0x0111, 0x0144, 0x0148, 0x00F3, 0x00F4, 0x0151, 0x00F6, 0x00F7,
        0x0159, 0x016F, 0x00FA, 0x0171, 0x00FC, 0x00FD, 0x0163, 0x02D9,
    ].map(|x| char::from_u32(x).unwrap_or_else(|| panic!("Unexpected value: {}. \
        Codepage entries should consist of valid Unicode characters only.", x)));
}

pub struct CodepageDecoder<'lut, R: Read> {
    /// A table for mapping all bytes with the most significant bit set to Unicode chars.
    decoding_lut: &'lut [char; 128],
    reader: R,
}

impl<'lut, R: Read> CodepageDecoder<'lut, R> {
    pub fn new(decoding_lut: &'lut [char; 128], reader: R) -> Self {
        Self {
            decoding_lut,
            reader,
        }
    }

    pub fn read(&mut self, buf: &mut [char]) -> std::io::Result<usize> {
        let mut read_byte = 0u8;
        let read_byte = std::slice::from_mut(&mut read_byte);
        for (i, entry) in buf.iter_mut().enumerate() {
            let bytes_read = self.reader.read(read_byte)?;
            if bytes_read == 0 {
                return Ok(i);
            }
            *entry = self.decode_byte(read_byte[0]);
        }
        Ok(buf.len())
    }

    pub fn read_single(&mut self) -> std::io::Result<char> {
        let mut read_byte = 0u8;
        let read_byte = std::slice::from_mut(&mut read_byte);
        let bytes_read = self.reader.read(read_byte)?;
        if bytes_read == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        Ok(self.decode_byte(read_byte[0]))
    }

    #[inline]
    fn decode_byte(&self, byte: u8) -> char {
        if byte < 128 {
            byte as char
        } else {
            self.decoding_lut[Into::<usize>::into(byte) - 128]
        }
    }
}

impl<'lut, R: Read> Iterator for CodepageDecoder<'lut, R> {
    type Item = std::io::Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        let read_byte = self.read_single();
        if read_byte
            .as_ref()
            .is_err_and(|err| err.kind() == std::io::ErrorKind::UnexpectedEof)
        {
            None
        } else {
            Some(read_byte)
        }
    }
}

pub struct CnvScanner<I: Iterator<Item = std::io::Result<char>>> {
    input: I,
    buffer: Vec<char>,
    pub current_element: Element<char>,
    next_position: Position,
}

impl<I: Iterator<Item = std::io::Result<char>>> CnvScanner<I> {
    const BUFFER_SIZE: usize = 2;

    pub fn new(input: I) -> Self {
        Self {
            input,
            buffer: Vec::with_capacity(Self::BUFFER_SIZE),
            current_element: Element::BeforeStream,
            next_position: Position::default(),
        }
    }

    fn refill_buffer(&mut self) -> std::io::Result<()> {
        while self.buffer.len() < Self::BUFFER_SIZE {
            let read_char = self.input.next();
            if read_char.is_none() {
                return Ok(());
            }
            self.buffer.push(read_char.unwrap()?);
        }
        Ok(())
    }

    fn match_newline(&mut self) -> MatchingResult {
        match self.buffer.as_slice() {
            ['\n', '\r', ..] => MatchingResult::Newline { length: 2 },
            ['\n', ..] => MatchingResult::Newline { length: 1 },
            ['\r', '\n', ..] => MatchingResult::Newline { length: 2 },
            ['\r', ..] => MatchingResult::Newline { length: 1 },
            ['\u{1e}', ..] => MatchingResult::Newline { length: 1 },
            _ => MatchingResult::Nothing,
        }
    }
}

impl<I: Iterator<Item = std::io::Result<char>>> PositionalIterator for CnvScanner<I> {
    type Item = char;

    fn advance(&mut self) -> std::io::Result<Element<Self::Item>> {
        self.refill_buffer()?;
        let new_element = if self.buffer.is_empty() {
            Element::AfterStream
        } else {
            let current_bounds = Bounds {
                start: self.next_position,
                end: self.next_position,
            };
            if let MatchingResult::Newline { length } = self.match_newline() {
                self.next_position = self.next_position.with_incremented_line(length);
                Element::WithinStream {
                    element: '\n',
                    bounds: current_bounds,
                }
            } else {
                self.next_position = self.next_position.with_incremented_column();
                Element::WithinStream {
                    element: self.buffer.remove(0),
                    bounds: current_bounds,
                }
            }
        };
        let superseded_element = std::mem::replace(&mut self.current_element, new_element);
        Ok(superseded_element)
    }

    fn get_current_element(&self) -> &Element<Self::Item> {
        &self.current_element
    }
}

enum MatchingResult {
    Nothing,
    Newline { length: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Position;
    use proptest::prelude::*;
    use test_case::test_case;

    fn make_iter_from(string: &str) -> impl Iterator<Item = std::io::Result<char>> + '_ {
        string.chars().map(|x| Ok(x)).into_iter()
    }

    #[test]
    fn output_should_be_empty_before_advancing_for_the_first_time() {
        let scanner = CnvScanner::new(make_iter_from("any input"));
        assert_eq!(scanner.get_current_element(), &Element::BeforeStream);
    }

    #[test]
    fn empty_input_should_result_in_no_output() {
        let mut scanner = CnvScanner::new(make_iter_from(""));
        scanner.advance().unwrap();
        assert_eq!(scanner.get_current_element(), &Element::AfterStream);
    }

    #[test_case("\n\r")]
    #[test_case("\n")]
    #[test_case("\r\n")]
    #[test_case("\r")]
    #[test_case("\x1e")]
    fn newline_sequences_should_be_recognized_correctly(input: &str) {
        let mut scanner = CnvScanner::new(input.chars().map(|x| Ok(x)));
        scanner.advance().unwrap();
        assert!(matches!(
            scanner.get_current_element(),
            &Element::WithinStream {
                element: '\n',
                bounds: _
            }
        ));
    }

    #[test_case("\n\rA")]
    #[test_case("\nA")]
    #[test_case("\r\nA")]
    #[test_case("\rA")]
    #[test_case("\x1eA")]
    fn newline_sequences_should_increment_line_position(input: &str) {
        let expected_position = Position {
            character: input.len() - 1,
            line: 2,
            column: 1,
        };
        let mut scanner = CnvScanner::new(input.chars().map(|x| Ok(x)));
        scanner.advance().unwrap();
        scanner.advance().unwrap();
        let &Element::WithinStream {
            element: _,
            bounds: current_bounds,
        } = scanner.get_current_element()
        else {
            panic!();
        };
        assert_eq!(current_bounds.start, expected_position);
        assert_eq!(current_bounds.end, expected_position);
    }

    proptest! {
        #[test]
        fn non_newline_characters_should_pass_through(alphanumeric in "[a-zA-Z0-9]") {
            let character = alphanumeric.chars().next().unwrap();
            let mut scanner = CnvScanner::new(std::iter::once(Ok(character)));
            scanner.advance().unwrap();
            let &Element::WithinStream {
                element: current_element,
                bounds: _,
            } = scanner.get_current_element()
            else {
                panic!();
            };
            assert_eq!(current_element, character);
        }

        #[test]
        fn non_newline_characters_should_increment_position_properly(alphanumeric in "[a-zA-Z0-9]") {
            let character = alphanumeric.chars().next().unwrap();
            let expected_position = Position { character: 1, line: 1, column: 2 };
            let mut scanner = CnvScanner::new(std::iter::once(Ok(character)));
            scanner.advance().unwrap();
            scanner.advance().unwrap();
            let &Element::WithinStream {
                element: _,
                bounds: current_bounds,
            } = scanner.get_current_element()
            else {
                panic!();
            };
            assert_eq!(current_bounds.start, expected_position);
            assert_eq!(current_bounds.end, expected_position);
        }
    }

    #[test]
    fn sequence_of_non_newline_characters_should_increment_position_properly() {
        let input = "abcd1234";
        let mut scanner = CnvScanner::new(input.chars().map(|x| Ok(x)));
        for i in 0..=input.len() {
            scanner.advance().unwrap();
            let &Element::WithinStream {
                element: _,
                bounds: current_bounds,
            } = scanner.get_current_element()
            else {
                panic!();
            };
            let expected_position = Position {
                character: i,
                line: 1,
                column: 1 + i,
            };
            assert_eq!(current_bounds.start, expected_position);
            assert_eq!(current_bounds.end, expected_position);
        }
    }

    #[test]
    fn sequence_of_non_newline_characters_should_be_passed_through_properly() {
        let input = "abcd1234";
        let mut scanner = CnvScanner::new(input.chars().map(|x| Ok(x)));
        for i in 0..input.len() {
            scanner.advance().unwrap();
            let &Element::WithinStream {
                element: current_element,
                bounds: _,
            } = scanner.get_current_element()
            else {
                panic!();
            };
            assert_eq!(current_element, input.chars().skip(i).next().unwrap());
        }
        scanner.advance().unwrap();
        assert_eq!(scanner.get_current_element(), &Element::AfterStream);
    }

    #[test]
    fn sequence_of_newline_characters_should_increment_position_properly() {
        let newlines = ["\n", "\n", "\n\r", "\n\r", "\r", "\r\n", "\x1e", "\x1e"];
        let input = newlines.join("");
        let mut scanner = CnvScanner::new(input.chars().map(|x| Ok(x)));
        for i in 0..=newlines.len() {
            scanner.advance().unwrap();
            let &Element::WithinStream {
                element: _,
                bounds: current_bounds,
            } = scanner.get_current_element()
            else {
                panic!();
            };
            let expected_position = Position {
                character: newlines.map(|x| x.len()).iter().take(i).sum(),
                line: 1 + i,
                column: 1,
            };
            assert_eq!(current_bounds.start, expected_position);
            assert_eq!(current_bounds.end, expected_position);
        }
    }

    #[test]
    fn sequence_of_newline_characters_should_be_interpreted_properly() {
        let newlines = ["\n", "\n", "\n\r", "\n\r", "\r", "\r\n", "\x1e", "\x1e"];
        let input = newlines.join("");
        let mut scanner = CnvScanner::new(input.chars().map(|x| Ok(x)));
        for _ in 0..=newlines.len() {
            scanner.advance().unwrap();
            assert!(matches!(scanner.get_current_element(), &Element::WithinStream {
                element: '\n',
                bounds: _,
            }));
        }
        scanner.advance().unwrap();
        assert_eq!(scanner.get_current_element(), &Element::AfterStream);
    }
}
