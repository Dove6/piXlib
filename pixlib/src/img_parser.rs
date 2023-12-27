use nom::{
    bytes::complete::tag,
    combinator::{map, map_res},
    error::{Error, ErrorKind},
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    Err, IResult, Needed,
};

use crate::formats_common::{ColorFormat, CompressionType, ImageData};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Header {
    pub width_px: u32,
    pub height_px: u32,
    pub color_format: ColorFormat,
    pub color_size_bytes: u32,
    pub compression_type: CompressionType,
    pub alpha_size_bytes: u32,
    pub x_position_px: i32,
    pub y_position_px: i32,
}

pub fn header(input: &[u8]) -> IResult<&[u8], Header> {
    map(
        tuple((
            tag("PIK\0"),
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            compression_type,
            le_u32,
            le_i32,
            le_i32,
        )),
        |(
            _,
            width_px,
            height_px,
            bit_depth,
            color_size_bytes,
            _,
            compression_type,
            alpha_size_bytes,
            x_position_px,
            y_position_px,
        )| {
            Header {
                width_px,
                height_px,
                color_format: ColorFormat::new(bit_depth),
                color_size_bytes,
                compression_type,
                alpha_size_bytes,
                x_position_px,
                y_position_px,
            }
        },
    )(input)
}

fn compression_type(input: &[u8]) -> IResult<&[u8], CompressionType> {
    map_res(le_u32, |compression_type| {
        Ok(match compression_type {
            0 => CompressionType::None,
            2 | 5 => CompressionType::Lzw2,
            4 => CompressionType::Jpeg,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}

fn image_data<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], ImageData> {
    let color_size = usize::try_from(header.color_size_bytes).unwrap();
    let alpha_size = usize::try_from(header.alpha_size_bytes).unwrap();
    let total_size = color_size + alpha_size;
    if input.len() < total_size {
        return Err(Err::Incomplete(Needed::new(total_size)));
    }

    let color = input[0..color_size].to_vec();
    let alpha = input[color_size..total_size].to_vec();
    Ok((&input[total_size..], ImageData { color, alpha }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImgFile {
    pub header: Header,
    pub image_data: ImageData,
}

pub fn parse_img(data: &[u8]) -> ImgFile {
    println!("Detected static image file.");
    let (data, header) = header(data).unwrap();
    println!("{:?}", header);
    let (_, image_data) = image_data(data, &header).unwrap();
    ImgFile { header, image_data }
}
