use std::{
    io::{Cursor, Write},
    sync::Arc,
};

use byteorder::{WriteBytesExt, LE};
use log::trace;
use nom::{
    bytes::complete::tag,
    combinator::map_res,
    error::{Error, ErrorKind},
    number::complete::{le_i32, le_u32},
    sequence::tuple,
    Err, IResult, Needed,
};

use crate::Rect;

use super::{ColorFormat, CompressionType, ImageData};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct ImgHeader {
    pub width_px: u32,
    pub height_px: u32,
    pub color_format: ColorFormat,
    pub color_size_bytes: u32,
    pub compression_type: CompressionType,
    pub alpha_size_bytes: u32,
    pub x_position_px: i32,
    pub y_position_px: i32,
}

pub fn header(input: &[u8]) -> IResult<&[u8], ImgHeader> {
    map_res(
        tuple((
            tag(b"PIK\0"),
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
            let color_format = ColorFormat::new(bit_depth)?;
            Ok::<_, String>(ImgHeader {
                width_px,
                height_px,
                color_format,
                color_size_bytes,
                compression_type,
                alpha_size_bytes,
                x_position_px,
                y_position_px,
            })
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

fn image_data<'a>(input: &'a [u8], header: &ImgHeader) -> IResult<&'a [u8], ImageData<'a>> {
    let color_size = usize::try_from(header.color_size_bytes).unwrap();
    let alpha_size = usize::try_from(header.alpha_size_bytes).unwrap();
    let total_size = color_size + alpha_size;
    if input.len() < total_size {
        return Err(Err::Incomplete(Needed::new(total_size)));
    }

    let color = &input[0..color_size];
    let alpha = &input[color_size..total_size];
    Ok((&input[total_size..], ImageData { color, alpha }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImgFile<'a> {
    pub header: ImgHeader,
    pub image_data: ImageData<'a>,
}

pub fn parse_img(data: &[u8]) -> Result<ImgFile, nom::Err<nom::error::Error<&[u8]>>> {
    trace!("Detected static image file.");
    let (data, header) = header(data)?;
    trace!("{:?}", header);
    let (_, image_data) = image_data(data, &header)?;
    Ok(ImgFile { header, image_data })
}

pub fn serialize_img(
    rgba8888: &[u8],
    rect: Rect,
    compression_type: CompressionType,
    color_format: ColorFormat,
) -> std::io::Result<Arc<Vec<u8>>> {
    trace!(
        "Serializing IMG ({:?}, {:?}, {:?})",
        rect,
        compression_type,
        color_format
    );
    let pixel_count = rect.get_width() * rect.get_height();
    let alpha_data = if rgba8888.iter().skip(3).step_by(4).any(|b| *b != 255) {
        Some(
            rgba8888
                .iter()
                .skip(3)
                .step_by(4)
                .copied()
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };
    let mut color_data = vec![255; pixel_count * 2];
    match color_format {
        ColorFormat::Rgb565 => {
            for i in 0..pixel_count {
                let r8: u16 = rgba8888[4 * i].into();
                let g8: u16 = rgba8888[4 * i + 1].into();
                let b8: u16 = rgba8888[4 * i + 2].into();
                let r5 = r8 * 31 / 255;
                let g6 = g8 * 63 / 255;
                let b5 = b8 * 31 / 255;
                let rgb565_l: u8 = (((g6 & 0x07) << 5) | b5).try_into().unwrap();
                let rgb565_h: u8 = ((r5 << 3) | ((g6 & 0x38) >> 3)).try_into().unwrap();
                color_data[2 * i] = rgb565_l;
                color_data[2 * i + 1] = rgb565_h;
            }
        }
        ColorFormat::Rgb555 => {
            for i in 0..pixel_count {
                let r8: u16 = rgba8888[4 * i].into();
                let g8: u16 = rgba8888[4 * i + 1].into();
                let b8: u16 = rgba8888[4 * i + 2].into();
                let r5 = r8 * 31 / 255;
                let g5 = g8 * 31 / 255;
                let b5 = b8 * 31 / 255;
                let rgb555_l: u8 = (((g5 & 0x07) << 5) | b5).try_into().unwrap();
                let rgb555_h: u8 = ((r5 << 2) | ((g5 & 0x18) >> 3)).try_into().unwrap();
                color_data[2 * i] = rgb555_l;
                color_data[2 * i + 1] = rgb555_h;
            }
        }
    }
    let (compression_type, color_data, alpha_data): (u32, _, _) = match compression_type {
        CompressionType::None => (0, color_data, alpha_data),
        CompressionType::Rle => return Err(std::io::ErrorKind::Unsupported.into()), // trully unsupported for IMG
        CompressionType::Lzw2 => return Err(std::io::ErrorKind::Unsupported.into()), // TODO: implement
        CompressionType::RleInLzw2 => return Err(std::io::ErrorKind::Unsupported.into()), // truly unsupported for IMG
        CompressionType::Jpeg => return Err(std::io::ErrorKind::Unsupported.into()), // TODO: implement
    };
    let alpha_len = alpha_data.as_ref().map(|a| a.len()).unwrap_or_default();
    let total_size = 40 + color_data.len() + alpha_len;
    let mut wrapped_vec = Arc::new(Vec::with_capacity(total_size));
    let vec = Arc::get_mut(&mut wrapped_vec).unwrap();
    let mut cur = Cursor::new(vec);
    cur.write_all(b"PIK\0")?;
    cur.write_u32::<LE>(rect.get_width().try_into().unwrap())?;
    cur.write_u32::<LE>(rect.get_height().try_into().unwrap())?;
    cur.write_u32::<LE>(match color_format {
        ColorFormat::Rgb565 => 16,
        ColorFormat::Rgb555 => 15,
    })?;
    cur.write_u32::<LE>(color_data.len().try_into().unwrap())?;
    cur.write_u32::<LE>(0)?;
    cur.write_u32::<LE>(compression_type)?;
    cur.write_u32::<LE>(alpha_len.try_into().unwrap())?;
    cur.write_i32::<LE>(rect.top_left_x.try_into().unwrap())?;
    cur.write_i32::<LE>(rect.top_left_y.try_into().unwrap())?;
    cur.write_all(&color_data)?;
    if let Some(alpha_data) = alpha_data {
        cur.write_all(&alpha_data)?;
    }
    Ok(wrapped_vec)
}
