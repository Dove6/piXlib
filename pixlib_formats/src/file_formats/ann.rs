use codepage_strings::{Coding, ConvertError};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{flat_map, map, map_res},
    error::{Error, ErrorKind},
    multi::{count, length_data},
    number::complete::{le_i16, le_u16, le_u32, le_u8},
    sequence::tuple,
    Err, IResult, Needed,
};

use super::{ColorFormat, CompressionType, ImageData};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub sprite_count: u16,
    pub color_format: ColorFormat,
    pub sequence_count: u16,
    pub short_description: String,
    pub frames_per_second: u32,
    pub unknown2: u32,
    pub opacity: u8,
    pub unknown3: u32,
    pub unknown4: u32,
    pub unknown5: u32,
    pub signature: String,
    pub unknown6: u32,
}

pub fn header(input: &[u8]) -> IResult<&[u8], Header> {
    map(
        tuple((
            alt((tag::<&str, &[u8], _>("NVM\0"), tag("NVP\0"))),
            le_u16,
            le_u16,
            le_u16,
            take(13u8),
            le_u32,
            le_u32,
            le_u8,
            le_u32,
            le_u32,
            le_u32,
            length_data(le_u32),
            le_u32,
        )),
        |(
            _,
            sprite_count,
            bit_depth,
            sequence_count,
            short_description,
            frames_per_second,
            unknown2,
            opacity,
            unknown3,
            unknown4,
            unknown5,
            signature,
            unknown6,
        )| {
            Header {
                sprite_count,
                color_format: ColorFormat::new(bit_depth as u32),
                sequence_count,
                short_description: from_cp1250(short_description, true).unwrap(),
                frames_per_second,
                unknown2,
                opacity,
                unknown3,
                unknown4,
                unknown5,
                signature: from_cp1250(signature, false).unwrap(),
                unknown6,
            }
        },
    )(input)
}

fn from_cp1250(mut v: &[u8], null_terminated: bool) -> Result<String, ConvertError> {
    if null_terminated {
        v = v.split(|c| *c == 0).next().unwrap();
    }
    Coding::new(1250)
        .unwrap()
        .decode(v)
        .and_then(|v| Ok(v.into_owned()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceHeader {
    pub name: String,
    pub frame_count: u16,
    pub unknown1: u16,
    pub unknown2: u32,
    pub loop_after: u32,
    pub unknown3: u32,
    pub unknown4: u32,
    pub unknown5: u16,
    pub opacity: u8,
    pub unknown6: u32,
    pub unknown7: u32,
    pub unknown8: u32,
    pub frame_to_sprite_mapping: Vec<u16>,
}

pub fn sequence_header(input: &[u8]) -> IResult<&[u8], SequenceHeader> {
    flat_map(
        tuple((
            take(32u8),
            le_u16,
            le_u16,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u16,
            le_u8,
            le_u32,
            le_u32,
            le_u32,
        )),
        |(
            name,
            frame_count,
            unknown1,
            unknown2,
            loop_after,
            unknown3,
            unknown4,
            unknown5,
            opacity,
            unknown6,
            unknown7,
            unknown8,
        )| {
            map(
                count(le_u16, frame_count as usize),
                move |frame_to_sprite_mapping| SequenceHeader {
                    name: from_cp1250(name, true).unwrap(),
                    frame_count,
                    unknown1,
                    unknown2,
                    loop_after,
                    unknown3,
                    unknown4,
                    unknown5,
                    opacity,
                    unknown6,
                    unknown7,
                    unknown8,
                    frame_to_sprite_mapping,
                },
            )
        },
    )(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub unknown1: u32,
    pub unknown2: u32,
    pub x_position_px: i16,
    pub y_position_px: i16,
    pub unknown3: u32,
    pub random_sfx_seed: u32,
    pub unknown4: u32,
    pub opacity: u8,
    pub unknown5: u8,
    pub unknown6: u32,
    pub name: String,
    pub random_sfx_list: Option<Vec<String>>,
}

pub fn random_sfx_list(
    random_sfx_seed: u32,
) -> impl Fn(&[u8]) -> IResult<&[u8], Option<Vec<String>>> {
    move |input| {
        if random_sfx_seed > 0 {
            map(length_data(le_u32), |random_sfx_list: &[u8]| {
                Some(
                    random_sfx_list
                        .split(|c| *c == 0)
                        .next()
                        .unwrap()
                        .split(|c| *c == b';')
                        .map(|sfx_name| from_cp1250(sfx_name, false).unwrap())
                        .collect(),
                )
            })(input)
        } else {
            Ok((input, None)) // not that simple (sometimes it's present despite seed being 0)
        }
    }
}

pub fn frame(input: &[u8]) -> IResult<&[u8], Frame> {
    flat_map(
        tuple((
            le_u32,
            le_u32,
            le_i16,
            le_i16,
            le_u32,
            le_u32,
            le_u32,
            le_u8,
            le_u8,
            le_u32,
            length_data(le_u32),
        )),
        |(
            unknown1,
            unknown2,
            x_position_px,
            y_position_px,
            unknown3,
            random_sfx_seed,
            unknown4,
            opacity,
            unknown5,
            unknown6,
            name,
        )| {
            map(random_sfx_list(random_sfx_seed), move |random_sfx_list| {
                Frame {
                    unknown1,
                    unknown2,
                    x_position_px,
                    y_position_px,
                    unknown3,
                    random_sfx_seed,
                    unknown4,
                    opacity,
                    unknown5,
                    unknown6,
                    name: from_cp1250(name, true).unwrap(),
                    random_sfx_list,
                }
            })
        },
    )(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteHeader {
    pub width_px: u16,
    pub height_px: u16,
    pub x_position_px: i16,
    pub y_position_px: i16,
    pub compression_type: CompressionType,
    pub color_size_bytes: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32,
    pub unknown4: u16,
    pub alpha_size_bytes: u32,
    pub name: String,
}

pub fn sprite_header(input: &[u8]) -> IResult<&[u8], SpriteHeader> {
    map(
        tuple((
            le_u16,
            le_u16,
            le_i16,
            le_i16,
            compression_type,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u16,
            le_u32,
            take(20u8),
        )),
        |(
            width_px,
            height_px,
            x_position_px,
            y_position_px,
            compression_type,
            color_size_bytes,
            unknown1,
            unknown2,
            unknown3,
            unknown4,
            alpha_size_bytes,
            name,
        )| {
            SpriteHeader {
                width_px,
                height_px,
                x_position_px,
                y_position_px,
                compression_type,
                color_size_bytes,
                unknown1,
                unknown2,
                unknown3,
                unknown4,
                alpha_size_bytes,
                name: from_cp1250(name, true).unwrap(),
            }
        },
    )(input)
}

fn compression_type(input: &[u8]) -> IResult<&[u8], CompressionType> {
    map_res(le_u16, |compression_type| {
        Ok(match compression_type {
            0 => CompressionType::None,
            2 => CompressionType::Lzw2,
            3 => CompressionType::RleInLzw2,
            4 => CompressionType::Rle,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}

fn image_data<'a>(input: &'a [u8], header: &SpriteHeader) -> IResult<&'a [u8], ImageData> {
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
pub struct Sequence {
    pub header: SequenceHeader,
    pub frames: Vec<Frame>,
}

pub fn sequences<'a>(mut input: &'a [u8], header: &Header) -> IResult<&'a [u8], Vec<Sequence>> {
    let mut sequences = Vec::with_capacity(header.sequence_count as usize);
    for _i in 0..header.sequence_count {
        let Ok((new_input, sequence_header)) = sequence_header(input) else {
            panic!();
        };
        input = new_input;
        // println!("Sequence #{_i} {:?}", sequence_header);
        let mut frames = Vec::with_capacity(sequence_header.frame_count as usize);
        for _j in 0..sequence_header.frame_count {
            let Ok((new_input, frame)) = frame(input) else {
                panic!();
            };
            input = new_input;
            // println!(" Frame #{_j} {:?}", frame);
            frames.push(frame);
        }
        sequences.push(Sequence {
            header: sequence_header,
            frames,
        });
    }
    Ok((input, sequences))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub header: SpriteHeader,
    pub image_data: ImageData,
}

pub fn sprites<'a>(mut input: &'a [u8], header: &Header) -> IResult<&'a [u8], Vec<Sprite>> {
    let mut sprite_headers = Vec::with_capacity(header.sprite_count as usize);
    let mut data_for_sprite = Vec::with_capacity(header.sprite_count as usize);
    for _i in 0..header.sprite_count {
        let Ok((new_input, sprite_header)) = sprite_header(input) else {
            panic!();
        };
        input = new_input;
        // println!("Sprite #{_i} {:?}", sprite_header);
        sprite_headers.push(sprite_header);
    }
    for i in 0..(header.sprite_count as usize) {
        let Ok((new_input, image_data)) = image_data(input, &sprite_headers[i]) else {
            panic!();
        };
        input = new_input;
        data_for_sprite.push(image_data);
    }
    Ok((
        input,
        sprite_headers
            .into_iter()
            .zip(data_for_sprite)
            .map(|(sprite_header, image_data)| Sprite {
                header: sprite_header,
                image_data,
            })
            .collect(),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnFile {
    pub header: Header,
    pub sequences: Vec<Sequence>,
    pub sprites: Vec<Sprite>,
}

pub fn parse_ann(data: &Vec<u8>) -> AnnFile {
    println!("Detected animation file.");
    let (data, header) = header(data).unwrap();
    println!("{:?}", header);
    let (data, sequences) = sequences(data, &header).unwrap();
    let (_, sprites) = sprites(data, &header).unwrap();
    AnnFile {
        header,
        sequences,
        sprites,
    }
}
