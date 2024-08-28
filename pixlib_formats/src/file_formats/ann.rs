use codepage_strings::ConvertError;
use log::trace;
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{cond, flat_map, map, map_res},
    error::{Error, ErrorKind},
    multi::{count, length_data},
    number::complete::{le_i16, le_u16, le_u32, le_u8},
    sequence::tuple,
    Err, IResult, Needed,
};

use super::{ColorFormat, CompressionType, DecodedStr, ImageData};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnHeader {
    pub sprite_count: u16,
    pub color_format: ColorFormat,
    pub sequence_count: u16,
    pub short_description: DecodedStr,
    pub frames_per_second: u32,
    pub unknown2: u32,
    pub opacity: u8,
    pub unknown3: u32,
    pub unknown4: u32,
    pub unknown5: u32,
    pub signature: DecodedStr,
    pub unknown6: u32,
}

pub fn header(input: &[u8]) -> IResult<&[u8], AnnHeader> {
    map(
        tuple((
            alt((tag(b"NVM\0"), tag(b"NVP\0"))),
            le_u16,
            map_res(le_u16, |bit_depth| ColorFormat::new(bit_depth.into())),
            le_u16,
            map_res(take(13u8), DecodedStr::from_bytes_null_terminated),
            le_u32,
            le_u32,
            le_u8,
            le_u32,
            le_u32,
            le_u32,
            map_res(length_data(le_u32), DecodedStr::from_bytes_null_terminated),
            le_u32,
        )),
        |(
            _,
            sprite_count,
            color_format,
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
            AnnHeader {
                sprite_count,
                color_format,
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
            }
        },
    )(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceHeader {
    pub name: DecodedStr,
    pub frame_count: u16,
    pub unknown1: u16,
    pub unknown2: u32,
    pub looping: LoopingSettings,
    pub unknown3: u32,
    pub unknown4: u32,
    pub unknown5: u16,
    pub opacity: u8,
    pub unknown6: u32,
    pub unknown7: u32,
    pub unknown8: u32,
    pub frame_to_sprite_mapping: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum LoopingSettings {
    LoopingAfter(usize),
    NoLooping,
}

impl LoopingSettings {
    fn new(value: u32) -> Self {
        if value == 0 {
            Self::NoLooping
        } else {
            Self::LoopingAfter(value.try_into().unwrap())
        }
    }
}

pub fn sequence_header(input: &[u8]) -> IResult<&[u8], SequenceHeader> {
    flat_map(
        tuple((
            take(32u8),
            le_u16,
            le_u16,
            le_u32,
            map(le_u32, LoopingSettings::new),
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
            looping,
            unknown3,
            unknown4,
            unknown5,
            opacity,
            unknown6,
            unknown7,
            unknown8,
        )| {
            map_res(
                count(le_u16, frame_count.into()),
                move |frame_to_sprite_mapping| {
                    Result::<_, ConvertError>::Ok(SequenceHeader {
                        name: DecodedStr::from_bytes_null_terminated(name)?,
                        frame_count,
                        unknown1,
                        unknown2,
                        looping,
                        unknown3,
                        unknown4,
                        unknown5,
                        opacity,
                        unknown6,
                        unknown7,
                        unknown8,
                        frame_to_sprite_mapping,
                    })
                },
            )
        },
    )(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameHeader {
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
    pub name: DecodedStr,
    pub random_sfx_list: Option<SeparatedDecodedStr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeparatedDecodedStr(Vec<String>, Option<Vec<u8>>);

impl SeparatedDecodedStr {
    pub fn rest(&self) -> &Option<Vec<u8>> {
        &self.1
    }

    pub fn with_rest(self, rest: Option<Vec<u8>>) -> Self {
        Self(self.0, rest)
    }

    pub fn is_totally_empty(&self) -> bool {
        self.0.is_empty() && self.1.as_ref().map(Vec::<u8>::is_empty).unwrap_or(true)
    }
}

impl DecodedStr {
    pub fn into_separated(self, separator: char) -> SeparatedDecodedStr {
        let DecodedStr(value, rest) = self;
        SeparatedDecodedStr(value.split(separator).map(str::to_owned).collect(), rest)
    }
}

impl AsRef<[String]> for SeparatedDecodedStr {
    fn as_ref(&self) -> &[String] {
        &self.0
    }
}

fn frame(input: &[u8]) -> IResult<&[u8], FrameHeader> {
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
            let has_random_sfx_list = random_sfx_seed > 0; // TODO: not that simple (sometimes it's present despite seed being 0)
            map_res(
                cond(
                    has_random_sfx_list,
                    map(
                        map_res(length_data(le_u32), DecodedStr::from_bytes_null_terminated),
                        move |random_sfx_list| random_sfx_list.into_separated(';'),
                    ),
                ),
                move |random_sfx_list| {
                    Result::<_, ConvertError>::Ok(FrameHeader {
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
                        name: DecodedStr::from_bytes_null_terminated(name)?,
                        random_sfx_list,
                    })
                },
            )
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
    pub name: DecodedStr,
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
            map_res(take(20u8), DecodedStr::from_bytes_null_terminated),
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
                name,
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

fn image_data<'a>(input: &'a [u8], header: &SpriteHeader) -> IResult<&'a [u8], ImageData<'a>> {
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
pub struct Sequence {
    pub header: SequenceHeader,
    pub frames: Vec<FrameHeader>,
}

pub fn sequences<'a>(mut input: &'a [u8], header: &AnnHeader) -> IResult<&'a [u8], Vec<Sequence>> {
    let mut sequences = Vec::with_capacity(header.sequence_count.into());
    for _i in 0..header.sequence_count {
        let Ok((new_input, sequence_header)) = sequence_header(input) else {
            panic!();
        };
        input = new_input;
        let mut frames = Vec::with_capacity(sequence_header.frame_count.into());
        for _j in 0..sequence_header.frame_count {
            let Ok((new_input, frame)) = frame(input) else {
                panic!();
            };
            input = new_input;
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
pub struct Sprite<'a> {
    pub header: SpriteHeader,
    pub image_data: ImageData<'a>,
}

pub fn sprites<'a>(mut input: &'a [u8], header: &AnnHeader) -> IResult<&'a [u8], Vec<Sprite<'a>>> {
    let mut sprite_headers = Vec::with_capacity(header.sprite_count.into());
    let mut data_for_sprite = Vec::with_capacity(header.sprite_count.into());
    for _ in 0..header.sprite_count {
        let Ok((new_input, sprite_header)) = sprite_header(input) else {
            panic!();
        };
        input = new_input;
        sprite_headers.push(sprite_header);
    }
    for sprite_header in sprite_headers.iter() {
        let Ok((new_input, image_data)) = image_data(input, sprite_header) else {
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
pub struct AnnFile<'a> {
    pub header: AnnHeader,
    pub sequences: Vec<Sequence>,
    pub sprites: Vec<Sprite<'a>>,
}

pub fn parse_ann(data: &[u8]) -> AnnFile {
    trace!("Detected animation file.");
    let (data, header) = header(data).unwrap();
    trace!("{:?}", header);
    let (data, sequences) = sequences(data, &header).unwrap();
    let (_, sprites) = sprites(data, &header).unwrap();
    AnnFile {
        header,
        sequences,
        sprites,
    }
}
