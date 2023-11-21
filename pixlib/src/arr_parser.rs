use std::fmt::Display;

use codepage_strings::{Coding, ConvertError};
use nom::{
    combinator::{map, flat_map},
    combinator::map_res,
    error::{Error, ErrorKind},
    multi::length_data,
    number::complete::le_i32,
    number::complete::le_u32,
    Err, IResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub size: u32,
}

pub fn header(input: &[u8]) -> IResult<&[u8], Header> {
    map(le_u32, |size| Header { size })(input)
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementData {
    Integer(i32),
    String(String),
    Boolean(bool),
    FixedPoint(i32),
}

impl Display for ElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementData::Integer(value) => f.write_fmt(format_args!("Integer: {}", value)),
            ElementData::String(value) => f.write_fmt(format_args!("String: {}", value)),
            ElementData::Boolean(value) => f.write_fmt(format_args!("Boolean: {}", value)),
            ElementData::FixedPoint(value) => f.write_fmt(format_args!("Integer: {}", Into::<f64>::into(*value) / 10000f64)),
        }
    }
}

pub fn element_data(tag_type: ElementType) -> impl Fn(&[u8]) -> IResult<&[u8], ElementData> {
    move |input| match tag_type {
        ElementType::Integer => map(le_i32, |value| ElementData::Integer(value))(input),
        ElementType::String => map(element_data_string, ElementData::String)(input),
        ElementType::Boolean => map(le_u32, |value| ElementData::Boolean(value == 1))(input),
        ElementType::FixedPoint => map(le_i32, |value| ElementData::FixedPoint(value))(input),
    }
}

pub fn element_data_string(input: &[u8]) -> IResult<&[u8], String> {
    map_res(length_data(le_u32), from_cp1250)(input)
}

fn from_cp1250(v: &[u8]) -> Result<String, ConvertError> {
    Coding::new(1250)
        .unwrap()
        .decode(v)
        .and_then(|v| Ok(v.into_owned()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub data_type: ElementType,
    pub data: ElementData,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrFile {
    pub header: Header,
    pub elements: Vec<Element>,
}

pub fn element(input: &[u8]) -> IResult<&[u8], Element> {
    flat_map(element_type, |data_type| {
        map(element_data(data_type), move |data| Element {
            data_type,
            data,
        })
    })(input)
}

pub fn describe_arr(data: &Vec<u8>) -> ArrFile {
    println!("Detected data array file.");
    let (mut data, header) = header(data).unwrap();
    println!("{:?}", header);
    let mut elements = Vec::<Element>::new();
    for _ in 0..header.size {
        let result = element(data).unwrap();
        data = result.0;
        elements.push(result.1);
    }
    println!("{:?}", elements);
    ArrFile { header, elements }
}
