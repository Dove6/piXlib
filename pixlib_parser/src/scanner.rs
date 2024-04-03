use crate::common::{Position, Spanned};

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

type IoReadResult = std::io::Result<u8>;
type ScannerInput = std::io::Result<char>;
type ScannerOutput = Spanned<char, Position, std::io::Error>;

pub struct CodepageDecoder<'lut, I: Iterator<Item = IoReadResult>> {
    /// A table for mapping all bytes with the most significant bit set to Unicode chars.
    decoding_lut: &'lut [char; 128],
    input: I,
}

impl<'lut, I: Iterator<Item = IoReadResult>> CodepageDecoder<'lut, I> {
    pub fn new(decoding_lut: &'lut [char; 128], input: I) -> Self {
        Self {
            decoding_lut,
            input,
        }
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

impl<'lut, I: Iterator<Item = IoReadResult>> Iterator for CodepageDecoder<'lut, I> {
    type Item = ScannerInput;

    fn next(&mut self) -> Option<Self::Item> {
        match self.input.next() {
            Some(Ok(byte)) => Some(Ok(self.decode_byte(byte))),
            Some(Err(err)) if err.kind() == std::io::ErrorKind::Interrupted => self.next(),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

pub struct CnvScanner<I: Iterator<Item = ScannerInput>> {
    input: I,
    buffer: Vec<char>,
    next_position: Position,
}

impl<I: Iterator<Item = ScannerInput>> CnvScanner<I> {
    const BUFFER_SIZE: usize = 2;

    pub fn new(input: I) -> Self {
        Self {
            input,
            buffer: Vec::with_capacity(Self::BUFFER_SIZE),
            next_position: Position::default(),
        }
    }

    fn refill_buffer(&mut self) -> std::io::Result<()> {
        while self.buffer.len() < Self::BUFFER_SIZE {
            match self.input.next() {
                Some(Ok(character)) => self.buffer.push(character),
                Some(Err(err)) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Some(Err(err)) => return Err(err),
                None => return Ok(()),
            };
        }
        Ok(())
    }

    fn match_newline(&mut self) -> Option<NewlineDetails> {
        match self.buffer.as_slice() {
            ['\n', '\r', ..] => Some(NewlineDetails { length: 2 }),
            ['\n', ..] => Some(NewlineDetails { length: 1 }),
            ['\r', '\n', ..] => Some(NewlineDetails { length: 2 }),
            ['\r', ..] => Some(NewlineDetails { length: 1 }),
            ['\u{1e}', ..] => Some(NewlineDetails { length: 1 }),
            _ => None,
        }
    }
}

impl<I: Iterator<Item = ScannerInput>> Iterator for CnvScanner<I> {
    type Item = ScannerOutput;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(err) = self.refill_buffer() {
            return Some(Err(err));
        }
        if self.buffer.is_empty() {
            None
        } else {
            let start = self.next_position;
            let character = if let Some(NewlineDetails { length }) = self.match_newline() {
                self.next_position = self.next_position.with_incremented_line(length);
                self.buffer.drain(..length);
                '\n'
            } else {
                self.next_position = self.next_position.with_incremented_column();
                self.buffer.remove(0)
            };
            Some(Ok((start, character, self.next_position)))
        }
    }
}

struct NewlineDetails {
    pub length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Position;
    use proptest::prelude::*;
    use test_case::test_case;

    fn iter_from(string: &str) -> impl Iterator<Item = std::io::Result<char>> + '_ {
        string.chars().map(|x| Ok(x)).into_iter()
    }

    fn into_iter_from(string: String) -> impl Iterator<Item = std::io::Result<char>> {
        string
            .chars()
            .map(|x| Ok(x))
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn empty_input_should_result_in_no_output() {
        let mut scanner = CnvScanner::new(iter_from(""));
        assert!(scanner.next().is_none());
    }

    #[test_case("\n\r")]
    #[test_case("\n")]
    #[test_case("\r\n")]
    #[test_case("\r")]
    #[test_case("\x1e")]
    fn newline_sequences_should_be_recognized_correctly(input: &str) {
        let mut scanner = CnvScanner::new(iter_from(input));
        assert_eq!(scanner.next().unwrap().unwrap().1, '\n');
        assert!(scanner.next().is_none());
    }

    #[test_case("\n\r")]
    #[test_case("\n")]
    #[test_case("\r\n")]
    #[test_case("\r")]
    #[test_case("\x1e")]
    fn newline_sequences_should_increment_line_position(input: &str) {
        let expected_position = Position {
            character: input.len(),
            line: 2,
            column: 1,
        };
        let mut scanner = CnvScanner::new(iter_from(input));
        assert_eq!(scanner.next().unwrap().unwrap().2, expected_position);
    }

    proptest! {
        #[test]
        fn non_newline_characters_should_pass_through(alphanumeric in "[a-zA-Z0-9]") {
            let expected_character = alphanumeric.chars().next().unwrap();
            let mut scanner = CnvScanner::new(iter_from(&alphanumeric));
            assert_eq!(scanner.next().unwrap().unwrap().1, expected_character);
            assert!(scanner.next().is_none());
        }

        #[test]
        fn non_newline_characters_should_increment_position_properly(alphanumeric in "[a-zA-Z0-9]") {
            let expected_position = Position { character: 1, line: 1, column: 2 };
            let mut scanner = CnvScanner::new(iter_from(&alphanumeric));
            assert_eq!(scanner.next().unwrap().unwrap().2, expected_position);
        }
    }

    #[test]
    fn sequence_of_non_newline_characters_should_be_passed_through_properly() {
        let input = "abcd1234";
        let mut scanner = CnvScanner::new(iter_from(input));
        for i in 0..input.len() {
            assert_eq!(
                scanner.next().unwrap().unwrap().1,
                input.chars().skip(i).next().unwrap()
            );
        }
        assert!(scanner.next().is_none());
    }

    #[test]
    fn sequence_of_non_newline_characters_should_increment_position_properly() {
        let input = "abcd1234";
        let mut scanner = CnvScanner::new(iter_from(input));
        for i in 0..input.len() {
            let expected_next_position = Position {
                character: i + 1,
                line: 1,
                column: 2 + i,
            };
            assert_eq!(scanner.next().unwrap().unwrap().2, expected_next_position);
        }
    }

    #[test]
    fn sequence_of_newline_characters_should_be_interpreted_properly() {
        let newlines = ["\n", "\n", "\n\r", "\n\r", "\r", "\r\n", "\x1e", "\x1e"];
        let input = newlines.join("");
        let mut scanner = CnvScanner::new(input.chars().map(|x| Ok(x)));
        for _ in 0..newlines.len() {
            assert_eq!(scanner.next().unwrap().unwrap().1, '\n');
        }
        assert!(scanner.next().is_none());
    }

    #[test]
    fn sequence_of_newline_characters_should_increment_position_properly() {
        let newlines = ["\n", "\n", "\n\r", "\n\r", "\r", "\r\n", "\x1e", "\x1e"];
        let mut scanner = CnvScanner::new(into_iter_from(newlines.join("")));
        for i in 0..newlines.len() {
            let expected_next_position = Position {
                character: newlines.map(|x| x.len()).iter().take(i + 1).sum(),
                line: 2 + i,
                column: 1,
            };
            assert_eq!(scanner.next().unwrap().unwrap().2, expected_next_position);
        }
    }

    #[test]
    fn io_error_should_be_passed_through_properly() {
        let expected_err_kind = std::io::ErrorKind::TimedOut;
        let mut scanner = CnvScanner::new(std::iter::once(Err(expected_err_kind.clone().into())));
        assert_eq!(
            scanner.next().unwrap().unwrap_err().kind(),
            expected_err_kind
        );
    }

    #[test]
    fn interrupted_error_kind_should_be_ignored() {
        let expected_character = 'a';
        let mut scanner = CnvScanner::new(
            [
                Err(std::io::ErrorKind::Interrupted.into()),
                Ok(expected_character),
            ]
            .into_iter(),
        );
        assert_eq!(scanner.next().unwrap().unwrap().1, expected_character);
    }
}
