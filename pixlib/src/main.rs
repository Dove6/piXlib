mod ann_parser;
mod arr_parser;
mod formats_common;
mod img_parser;
mod lzw2_decoder;
mod rle_decoder;

use ann_parser::AnnFile;
use arr_parser::ArrFile;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    prelude::{
        default, App, Assets, Camera, Camera2dBundle, Color, Commands, Gizmos, GlobalTransform,
        Image, PluginGroup, Query, ResMut, Startup, Transform, Update,
    },
    render::{
        render_resource::{Extent3d, TextureFormat},
        texture::ImagePlugin,
    },
    sprite::{Anchor, Sprite, SpriteBundle},
    window::{Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use formats_common::{ColorFormat, CompressionType, ImageData};
use img_parser::ImgFile;
use opticaldisc::iso::IsoFs;

use crate::{ann_parser::parse_ann, arr_parser::parse_arr, img_parser::parse_img};
use crate::{lzw2_decoder::decode_lzw2, rle_decoder::decode_rle};
use std::{
    fs::{self, File},
    io::Read,
    iter,
};

const WINDOW_SIZE: (f32, f32) = (800., 600.);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WINDOW_SIZE.into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let parsed_file = parse_file();
    let (transform, image) = match parsed_file {
        AmFile::Img(img_file) => (
            Transform::from_xyz(
                img_file.header.x_position_px as f32,
                -img_file.header.y_position_px as f32,
                0.0,
            ),
            image_data_to_image(
                &img_file.image_data,
                (img_file.header.width_px, img_file.header.height_px),
                img_file.header.color_format,
                img_file.header.compression_type,
                img_file.header.alpha_size_bytes > 0,
            ),
        ),
        AmFile::Ann(ann_file) => {
            let sprite = &ann_file.sprites[0];
            (
                Transform::from_xyz(
                    sprite.header.x_position_px as f32,
                    -sprite.header.y_position_px as f32,
                    0.0,
                ),
                image_data_to_image(
                    &sprite.image_data,
                    (
                        sprite.header.width_px as u32,
                        sprite.header.height_px as u32,
                    ),
                    ann_file.header.color_format,
                    sprite.header.compression_type,
                    sprite.header.alpha_size_bytes > 0,
                ),
            )
        }
        _ => panic!(),
    };
    let texture = images.add(image);
    println!("Transform: {:?}", transform);

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(WINDOW_SIZE.0 / 2.0, WINDOW_SIZE.1 / -2.0, 0.0),
        tonemapping: Tonemapping::None,
        deband_dither: DebandDither::Disabled,
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture,
        sprite: Sprite {
            anchor: Anchor::TopLeft,
            ..default()
        },
        transform,
        ..default()
    });
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };
    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    gizmos.circle_2d(point, 10., Color::WHITE);
}

fn parse_file() -> AmFile {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        panic!("Usage: iso_browser path_to_iso path_to_file_on_iso [output_path]");
    }
    let path_to_iso = args[1].clone();
    let path_to_file = args[2].to_ascii_uppercase();
    let output_path = args.get(3);

    let iso_file = File::open(&path_to_iso).unwrap();
    let mut iso = read_iso(&iso_file);
    parse_file_from_iso(&mut iso, &path_to_file, output_path.map(|v| v.as_ref()))
}

fn image_data_to_image(
    image_data: &ImageData,
    image_size_px: (u32, u32),
    color_format: ColorFormat,
    compression_type: CompressionType,
    has_alpha: bool,
) -> Image {
    let color_data = match compression_type {
        CompressionType::None => image_data.color.to_owned(),
        CompressionType::Rle => decode_rle(&image_data.color, 2),
        CompressionType::Lzw2 => decode_lzw2(&image_data.color),
        CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(&image_data.color), 2),
        _ => panic!(),
    };
    let alpha_data = match compression_type {
        _ if !has_alpha => vec![],
        CompressionType::None => image_data.alpha.to_owned(),
        CompressionType::Rle => decode_rle(&image_data.alpha, 1),
        CompressionType::Lzw2 | CompressionType::Jpeg => decode_lzw2(&image_data.alpha),
        CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(&image_data.alpha), 1),
    };
    let color_converter = match color_format {
        ColorFormat::Rgb565 => rgb565_to_rgb888,
        ColorFormat::Rgb555 => rgb555_to_rgb888,
    };
    let converted_image = color_data
        .chunks_exact(2)
        .map(|rgb565| rgb565.try_into().unwrap())
        .map(color_converter)
        .zip(alpha_data.iter().chain(iter::repeat(&255)))
        .map(|([r, g, b], a)| [r, g, b, *a])
        .flatten()
        .collect();
    Image::new(
        Extent3d {
            width: image_size_px.0 as u32,
            height: image_size_px.1 as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        converted_image,
        TextureFormat::Rgba8Unorm,
    )
}

fn rgb565_to_rgb888(rgb565: [u8; 2]) -> [u8; 3] {
    let rgb565 = u16::from_le_bytes(rgb565);
    let r5 = (rgb565 >> 11) & 0x1f;
    let g6 = (rgb565 >> 5) & 0x3f;
    let b5 = rgb565 & 0x1f;
    [
        (r5 * 255 / 31) as u8,
        (g6 * 255 / 63) as u8,
        (b5 * 255 / 31) as u8,
    ]
}

fn rgb555_to_rgb888(rgb555: [u8; 2]) -> [u8; 3] {
    let rgb555 = u16::from_le_bytes(rgb555);
    let r5 = (rgb555 >> 10) & 0x1f;
    let g5 = (rgb555 >> 5) & 0x1f;
    let b5 = rgb555 & 0x1f;
    [
        (r5 * 255 / 31) as u8,
        (g5 * 255 / 31) as u8,
        (b5 * 255 / 31) as u8,
    ]
}

fn read_iso(iso_file: &File) -> IsoFs<&File> {
    let mut iso = opticaldisc::iso::IsoFs::new(iso_file).unwrap();

    println!("Loaded ISO file.");
    for entry in iso.read_dir("/").unwrap().iter() {
        println!(
            "Entry discovered: {}, is file? {}",
            &entry.name(),
            entry.is_file()
        );
    }

    iso
}

fn parse_file_from_iso(
    iso: &mut IsoFs<&File>,
    filename: &str,
    output_filename: Option<&str>,
) -> AmFile {
    let mut buffer = Vec::<u8>::new();
    let bytes_read = iso
        .open_file(&filename)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    println!("Read file {} ({} bytes)", filename, bytes_read);

    if let Some(output_path) = output_filename {
        fs::write(&output_path, &buffer).expect("Could not write file");
    }

    let extension = filename
        .split('/')
        .last()
        .unwrap()
        .split('.')
        .last()
        .unwrap();

    match extension {
        "ANN" => AmFile::Ann(parse_ann(&buffer)),
        "ARR" => AmFile::Arr(parse_arr(&buffer)),
        "CLASS" | "CNV" | "DEF" => {
            println!("Detected script file.");
            AmFile::None
        }
        "DTA" => {
            println!("Detected text database file.");
            AmFile::None
        }
        "FLD" => {
            println!("Detected numerical matrix file.");
            AmFile::None
        }
        "FNT" => {
            println!("Detected font file.");
            AmFile::None
        }
        "IMG" => AmFile::Img(parse_img(&buffer)),
        "INI" => {
            println!("Detected text configuration file.");
            AmFile::None
        }
        "LOG" => {
            println!("Detected log file.");
            AmFile::None
        }
        "SEQ" => {
            println!("Detected animation sequence description file.");
            AmFile::None
        }
        "WAV" => {
            println!("Detected audio file.");
            AmFile::None
        }
        _ => {
            println!("Unknown file type!");
            AmFile::None
        }
    }
}

enum AmFile {
    Ann(AnnFile),
    Arr(ArrFile),
    Img(ImgFile),
    None,
}
