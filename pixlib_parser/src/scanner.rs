use std::collections::VecDeque;

use lazy_static::lazy_static;
use regex::bytes::Regex;

use crate::{
    common::{Position, Spanned},
    declarative_parser,
};

type IoReadResult = std::io::Result<u8>;
type ScannerInput = std::io::Result<char>;
type ScannerOutput = Spanned<char, Position, std::io::Error>;

#[derive(Debug, Clone)]
pub struct CnvFile(pub Vec<char>);

impl AsRef<[char]> for CnvFile {
    fn as_ref(&self) -> &[char] {
        &self.0
    }
}

impl CnvFile {
    pub fn as_parser_input(&self) -> impl Iterator<Item = declarative_parser::ParserInput> + '_ {
        self.0.iter().enumerate().map(|(i, c)| {
            Ok((
                Position {
                    line: 1,
                    column: 1 + i,
                    character: i,
                },
                *c,
                Position {
                    line: 1,
                    column: 2 + i,
                    character: i + 1,
                },
            ))
        })
    }
}

pub fn parse_cnv(input: &[u8]) -> CnvFile {
    let mut input = input.iter().map(|b| Ok(*b)).peekable();
    let mut first_line = Vec::<u8>::new();
    while let Some(res) =
        input.next_if(|res| res.as_ref().is_ok_and(|c| !matches!(c, b'\r' | b'\n')))
    {
        first_line.push(res.unwrap())
    }
    while let Some(res) =
        input.next_if(|res| res.as_ref().is_ok_and(|c| matches!(c, b'\r' | b'\n')))
    {
        first_line.push(res.unwrap())
    }
    let input: Box<dyn Iterator<Item = std::io::Result<u8>>> = match CnvHeader::try_new(&first_line)
    {
        Ok(Some(CnvHeader {
            cipher_class: _,
            step_count,
        })) => Box::new(CnvDecoder::new(
            input.collect::<Vec<_>>().into_iter(),
            step_count,
        )),
        Ok(None) => Box::new(
            first_line
                .into_iter()
                .map(Ok)
                .chain(input.collect::<Vec<_>>()),
        ),
        Err(err) => panic!("{}", err),
    };
    let decoder = CodepageDecoder::new(&CP1250_LUT, input);
    let scanner = CnvScanner::new(decoder);
    CnvFile(scanner.map(|r| r.unwrap().1).collect())
}

#[derive(Clone, Debug)]
struct IterBuf<I: Iterator<Item = IoReadResult>> {
    input: I,
    buffer: VecDeque<u8>,
    capacity: usize,
}

impl<I: Iterator<Item = IoReadResult>> IterBuf<I> {
    pub fn with_capacity(capacity: usize, input: I) -> Self {
        Self {
            input,
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn refill(&mut self) -> std::io::Result<()> {
        while self.buffer.len() < self.capacity {
            if let Some(next) = self.input.next() {
                self.buffer.push_back(next?);
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn peek(&self) -> Option<u8> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer[0])
        }
    }

    pub fn advance(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            if let Some(result) = self.input.next() {
                result?;
            }
        } else {
            self.buffer.pop_front();
        }
        self.refill()?;
        Ok(())
    }

    pub fn advance_n(&mut self, n: usize) -> std::io::Result<()> {
        let old_len = self.buffer.len();
        if n >= old_len {
            self.buffer.clear();
            for _ in 0..(n - old_len) {
                if let Some(result) = self.input.next() {
                    result?;
                } else {
                    break;
                }
            }
        } else {
            for _ in 0..n {
                self.buffer.pop_front();
            }
        }
        self.refill()?;
        Ok(())
    }

    pub fn starts_with(&self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return true;
        }
        if self.buffer.len() < bytes.len() {
            return false;
        }
        let slices = self.buffer.as_slices();
        if bytes.len() <= slices.0.len() {
            slices.0[..bytes.len()] == *bytes
        } else {
            *slices.0 == bytes[..slices.0.len()]
                && slices.1[..(bytes.len() - slices.0.len())] == bytes[slices.0.len()..]
        }
    }
}

impl<I: Iterator<Item = IoReadResult>> PartialEq<&[u8]> for IterBuf<I> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.buffer == *other
    }
}

#[cfg(test)]
mod buf_iter_tests {
    use super::*;

    #[test]
    fn buffer_should_be_empty_just_after_creation() {
        let input = "1234567890";
        let capacity = 2;
        let iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));

        assert!(iter_buf.is_empty());
        assert_eq!(iter_buf.peek(), None);
    }

    #[test]
    fn refill_should_refill_the_buffer_if_possible() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        assert!(iter_buf.refill().is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'1'));
        assert_eq!(iter_buf, b"12".as_ref());
    }

    #[test]
    fn advance_should_move_full_buffer_by_one_byte() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        iter_buf.refill().unwrap();
        assert!(iter_buf.advance().is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'2'));
        assert_eq!(iter_buf, b"23".as_ref());
    }

    #[test]
    fn advance_should_skip_one_byte_and_refill_if_the_buffer_is_empty() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        assert!(iter_buf.advance().is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'2'));
        assert_eq!(iter_buf, b"23".as_ref());
    }

    #[test]
    fn advance_n_with_n_equal_to_1_should_work_just_as_advance_for_full_buffer() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        iter_buf.refill().unwrap();
        assert!(iter_buf.advance_n(1).is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'2'));
        assert_eq!(iter_buf, b"23".as_ref());
    }

    #[test]
    fn advance_n_with_n_equal_to_1_should_work_just_as_advance_for_empty_buffer() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        assert!(iter_buf.advance_n(1).is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'2'));
        assert_eq!(iter_buf, b"23".as_ref());
    }

    #[test]
    fn advance_n_should_work_correctly_for_n_equal_to_capacity_and_full_buffer() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        iter_buf.refill().unwrap();
        assert!(iter_buf.advance_n(capacity).is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'3'));
        assert_eq!(iter_buf, b"34".as_ref());
    }

    #[test]
    fn advance_n_should_work_correctly_for_n_equal_to_capacity_and_empty_buffer() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        assert!(iter_buf.advance_n(capacity).is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'3'));
        assert_eq!(iter_buf, b"34".as_ref());
    }

    #[test]
    fn advance_n_should_work_correctly_for_n_above_capacity() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        iter_buf.refill().unwrap();
        assert!(iter_buf.advance_n(3).is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'4'));
        assert_eq!(iter_buf, b"45".as_ref());
    }

    #[test]
    fn advance_n_should_work_correctly_for_n_above_capacity_and_empty_buffer() {
        let input = "1234567890";
        let capacity = 2;
        let mut iter_buf = IterBuf::with_capacity(capacity, input.bytes().map(Ok));
        assert!(iter_buf.advance_n(3).is_ok());

        assert!(!iter_buf.is_empty());
        assert_eq!(iter_buf.len(), capacity);
        assert_eq!(iter_buf.peek(), Some(b'4'));
        assert_eq!(iter_buf, b"45".as_ref());
    }
}

#[derive(Debug, Clone)]
pub struct CnvHeader {
    pub cipher_class: char,
    pub step_count: usize,
}

lazy_static! {
    static ref CNV_HEADER_REGEX: Regex = Regex::new(r"^\{<([CD]):(\d{1,10})>\}\s+$")
        .expect("The regex for CNV header should be defined correctly.");
}

impl CnvHeader {
    pub fn try_new(line: &[u8]) -> Result<Option<Self>, &'static str> {
        if let Some(m) = CNV_HEADER_REGEX.captures(line) {
            let cipher_class = m[1][0] as char;
            let step_count = m[2]
                .iter()
                .fold(0, |acc, digit| acc * 10 + (digit - b'0') as usize);
            Ok(Some(Self {
                cipher_class,
                step_count,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct CnvDecoderState {
    position: usize,
    current_step: u16,
    newline_counter: u8,
}

#[derive(Clone, Debug)]
pub struct CnvDecoder<I: Iterator<Item = IoReadResult>> {
    input: IterBuf<I>,
    steps: u16,
    state: CnvDecoderState,
}

static NEWLINE_TOKEN: &[u8; 3] = b"<E>";

impl<I: Iterator<Item = IoReadResult>> CnvDecoder<I> {
    #[allow(dead_code)]
    pub fn new(input: I, steps: usize) -> Self {
        Self {
            input: IterBuf::with_capacity(NEWLINE_TOKEN.len(), input),
            steps: steps as u16,
            state: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn get_position(&self) -> usize {
        self.state.position
    }

    fn try_consume_newline(&mut self) -> std::io::Result<bool> {
        if self.input.starts_with(NEWLINE_TOKEN.as_ref()) {
            self.state.position += NEWLINE_TOKEN.len();
            self.state.newline_counter += 1;
            self.input.advance_n(NEWLINE_TOKEN.len())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn try_decode_byte_and_advance(&mut self) -> std::io::Result<Option<u8>> {
        let shift = ((self.state.current_step >> 1) + 1) as isize;
        let shift = shift
            * if self.state.current_step % 2 == 0 {
                -1
            } else {
                1
            };
        let Some(decoded_byte) = self.input.peek() else {
            return Ok(None);
        };
        let decoded_byte = decoded_byte as isize + shift;
        self.state.current_step = (self.state.current_step + 1) % self.steps;
        self.input.advance()?;
        Ok(Some(decoded_byte as u8))
    }
}

impl<I: Iterator<Item = IoReadResult>> Iterator for CnvDecoder<I> {
    type Item = IoReadResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            if let Err(e) = self.input.refill() {
                return Some(Err(e));
            }
            if self.input.is_empty() {
                return None;
            }
        }
        match self.try_consume_newline() {
            Err(e) => return Some(Err(e)),
            Ok(true) => {
                while matches!(self.input.peek(), Some(b'\n') | Some(b'\r')) {
                    self.state.position += 1;
                    if let Err(e) = self.input.advance() {
                        return Some(Err(e));
                    }
                }
                if self.state.newline_counter == 6 {
                    self.state.newline_counter = 0;
                }
                return Some(Ok(b'\n'));
            }
            Ok(false) => (),
        }
        match self.try_decode_byte_and_advance() {
            Err(e) => Some(Err(e)),
            Ok(opt) => opt.map(Ok),
        }
    }
}

#[cfg(test)]
mod cnv_decoder_tests {
    use super::*;

    #[test]
    fn simple_text_should_be_decoded_correctly() {
        let input = b"PALCFQ".as_ref();
        let decoder = CnvDecoder::new(input.iter().map(|b| Ok(*b)), 6);
        assert_eq!(
            decoder.map(|res| res.unwrap()).collect::<Vec<_>>(),
            b"OBJECT"
        );
    }

    #[test]
    fn simple_text_with_newline_marker_should_be_decoded_correctly() {
        let input = b"PAL<E>CFQ".as_ref();
        let decoder = CnvDecoder::new(input.iter().map(|b| Ok(*b)), 6);
        assert_eq!(
            decoder.map(|res| res.unwrap()).collect::<Vec<_>>(),
            b"OBJ\nECT"
        );
    }

    #[test]
    fn simple_text_with_newline_marker_and_newlines_should_be_decoded_correctly() {
        let input = b"PAL<E>\r\nCFQ".as_ref();
        let decoder = CnvDecoder::new(input.iter().map(|b| Ok(*b)), 6);
        assert_eq!(
            decoder.map(|res| res.unwrap()).collect::<Vec<_>>(),
            b"OBJ\nECT"
        );
    }

    #[test]
    fn newlines_without_newline_marker_preceding_them_should_be_decoded_literally() {
        let input = b"PAL\r\nC".as_ref();
        let decoder = CnvDecoder::new(input.iter().map(|b| Ok(*b)), 6);
        assert_eq!(
            decoder.map(|res| res.unwrap()).collect::<Vec<_>>(),
            b"OBJ\x0f\x07F"
        );
    }

    #[test]
    fn step_count_should_be_reset_after_reaching_max_step() {
        let input = b"BKC\x1eP>!JQRD".as_ref();
        let decoder = CnvDecoder::new(input.iter().map(|b| Ok(*b)), 6);
        assert_eq!(
            decoder.map(|res| res.unwrap()).collect::<Vec<_>>(),
            b"ALA MA KOTA"
        );
    }

    #[test]
    fn several_newline_markers_should_be_decoded_correctly() {
        let input = b"A<E><E><E><E>B".as_ref();
        let decoder = CnvDecoder::new(input.iter().map(|b| Ok(*b)), 6);
        assert_eq!(
            decoder.map(|res| res.unwrap()).collect::<Vec<_>>(),
            b"@\n\n\n\nC"
        );
    }
}

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
        string.chars().map(Ok)
    }

    fn into_iter_from(string: String) -> impl Iterator<Item = std::io::Result<char>> {
        string.chars().map(Ok).collect::<Vec<_>>().into_iter()
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
                input.chars().nth(i).unwrap()
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
        let mut scanner = CnvScanner::new(input.chars().map(Ok));
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
        let mut scanner = CnvScanner::new(std::iter::once(Err(expected_err_kind.into())));
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
