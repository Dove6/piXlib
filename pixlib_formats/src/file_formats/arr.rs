use std::{
    fmt::Display,
    io::{Cursor, Write},
    sync::Arc,
};

use byteorder::{WriteBytesExt, LE};
use log::trace;
use nom::{
    combinator::map_res,
    combinator::{flat_map, map},
    error::{Error, ErrorKind},
    multi::length_data,
    number::complete::le_i32,
    number::complete::le_u32,
    Err, IResult,
};

use super::DecodedStr;

const FIXED_POINT_SCALE: f64 = 10000f64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrHeader {
    pub size: u32,
}

pub fn header(input: &[u8]) -> IResult<&[u8], ArrHeader> {
    map(le_u32, |size| ArrHeader { size })(input)
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum ElementType {
    Integer,
    String,
    Boolean,
    FixedPoint,
}

fn element_type(input: &[u8]) -> IResult<&[u8], ElementType> {
    map_res(le_u32, |element_type| {
        Ok(match element_type {
            1 => ElementType::Integer,
            2 => ElementType::String,
            3 => ElementType::Boolean,
            4 => ElementType::FixedPoint,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}

#[derive(Clone, Debug)]
pub enum ElementData {
    Integer(i32),
    String(DecodedStr),
    Boolean(bool),
    FixedPoint(f64),
}

impl PartialEq for ElementData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::FixedPoint(l0), Self::FixedPoint(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for ElementData {}

impl Display for ElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementData::Integer(value) => f.write_fmt(format_args!("Integer: {}", value)),
            ElementData::String(value) => f.write_fmt(format_args!("String: {}", value.0)),
            ElementData::Boolean(value) => f.write_fmt(format_args!("Boolean: {}", value)),
            ElementData::FixedPoint(value) => f.write_fmt(format_args!("FixedPoint: {}", value)),
        }
    }
}

pub fn element_data(tag_type: ElementType) -> impl Fn(&[u8]) -> IResult<&[u8], ElementData> {
    move |input| match tag_type {
        ElementType::Integer => map(le_i32, ElementData::Integer)(input),
        ElementType::String => map(
            map_res(length_data(le_u32), DecodedStr::from_bytes),
            ElementData::String,
        )(input),
        ElementType::Boolean => map(le_u32, |value| ElementData::Boolean(value == 1))(input),
        ElementType::FixedPoint => map(le_i32, |value| {
            ElementData::FixedPoint(Into::<f64>::into(value) / FIXED_POINT_SCALE)
        })(input),
    }
}

pub type ArrFile = Vec<ElementData>;

pub fn element(input: &[u8]) -> IResult<&[u8], ElementData> {
    flat_map(element_type, |data_type| element_data(data_type))(input)
}

pub fn parse_arr(data: &[u8]) -> ArrFile {
    trace!("Detected data array file.");
    let (mut data, header) = header(data).unwrap();
    trace!("{:?}", header);
    let mut elements = Vec::<ElementData>::new();
    for _ in 0..header.size {
        let result = element(data).unwrap();
        data = result.0;
        elements.push(result.1);
    }
    trace!("{:?}", elements);
    elements
}

pub fn serialize_arr(arr: &[ElementData]) -> std::io::Result<Arc<Vec<u8>>> {
    let total_size = 4 + arr
        .iter()
        .map(|e| match e {
            ElementData::String(s) => 4 + s.total_length(),
            _ => 4,
        })
        .sum::<usize>();
    let mut wrapped_vec = Arc::new(Vec::with_capacity(total_size));
    let vec = Arc::get_mut(&mut wrapped_vec).unwrap();
    let mut cur = Cursor::new(vec);
    cur.write_u32::<LE>(arr.len() as u32)?;
    for e in arr.iter() {
        match e {
            ElementData::Integer(i) => {
                cur.write_i32::<LE>(1)?;
                cur.write_i32::<LE>(*i)?;
            }
            ElementData::String(s) => {
                cur.write_i32::<LE>(2)?;
                cur.write_u32::<LE>(s.total_length() as u32)?;
                cur.write(&s.clone().to_bytes().map_err(|e| std::io::Error::other(e))?)?;
            }
            ElementData::Boolean(b) => {
                cur.write_i32::<LE>(3)?;
                cur.write_u32::<LE>(if *b { 1 } else { 0 })?;
            }
            ElementData::FixedPoint(d) => {
                cur.write_i32::<LE>(4)?;
                cur.write_i32::<LE>((d * FIXED_POINT_SCALE) as i32)?;
            }
        }
    }
    Ok(wrapped_vec)
}

#[cfg(test)]
mod test_arr_serialization {
    use super::*;

    #[test]
    fn should_deserialize_correctly() {
        assert_eq!(
            parse_arr(&[
                0x05, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0x00,
                0x00, 0x00, 0x02, 0x00, 0x00, 0x00, b'h', b'i', 0x02, 0x00, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x00, b'h', b'\0', b'k', 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x04, 0x00, 0x00, 0x00, 0xA8, 0x7A, 0x00, 0x00,
            ]),
            &[
                ElementData::Integer(-1),
                ElementData::String(DecodedStr("hi".to_owned(), None)),
                ElementData::String(DecodedStr("h\0k".to_owned(), None)),
                ElementData::Boolean(true),
                ElementData::FixedPoint(3.14),
            ]
        );
    }

    #[test]
    fn should_serialize_correctly() {
        assert_eq!(
            serialize_arr(&[
                ElementData::Integer(-1),
                ElementData::String(DecodedStr("hi".to_owned(), None)),
                ElementData::String(DecodedStr("h".to_owned(), Some(vec![b'k']))),
                ElementData::Boolean(true),
                ElementData::FixedPoint(3.14),
            ])
            .unwrap()
            .as_ref(),
            &vec![
                0x05, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0x00,
                0x00, 0x00, 0x02, 0x00, 0x00, 0x00, b'h', b'i', 0x02, 0x00, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x00, b'h', b'\0', b'k', 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x04, 0x00, 0x00, 0x00, 0xA8, 0x7A, 0x00, 0x00,
            ]
        );
    }
}
