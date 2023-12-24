mod ann_parser;
mod arr_parser;
mod img_parser;
mod lzw2_decoder;

use ann_parser::AnnFile;
use arr_parser::ArrFile;
use bevy::{
    prelude::{
        default, App, Assets, Camera, Camera2dBundle, Color, Commands, Gizmos, GlobalTransform,
        Image, PluginGroup, Query, ResMut, Startup, Transform, Update,
    },
    render::render_resource::{Extent3d, TextureFormat},
    sprite::{Anchor, Sprite, SpriteBundle},
    window::{Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use img_parser::ImgFile;
use opticaldisc::iso::IsoFs;
use rgb565::Rgb565;

use crate::{img_parser::parse_img, lzw2_decoder::decode_lzw2};
use crate::{ann_parser::parse_ann, arr_parser::parse_arr};
use std::{
    fs::{self, File},
    io::Read,
    iter,
};

const WINDOW_SIZE: (f32, f32) = (800., 600.);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WINDOW_SIZE.into(),
                ..default()
            }),
            ..default()
        }))
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
            img_to_image(&img_file),
        ),
        AmFile::Ann(ann_file) => {
            let sprite = &ann_file.sprites[0];
            (
                Transform::from_xyz(
                    sprite.header.x_position_px as f32,
                    -sprite.header.y_position_px as f32,
                    0.0,
                ),
                ann_sprite_to_image(sprite),
            )
        }
        _ => panic!(),
    };
    let texture = images.add(image);
    println!("Transform: {:?}", transform);

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(WINDOW_SIZE.0 / 2.0, WINDOW_SIZE.1 / -2.0, 0.0),
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

fn img_to_image(img_file: &ImgFile) -> Image {
    let color_data = match img_file.header.compression_type {
        img_parser::CompressionType::None => img_file.image_data.color.to_owned(),
        img_parser::CompressionType::Clzw2 => decode_lzw2(&img_file.image_data.color),
        _ => panic!(),
    };
    let has_alpha = img_file.header.alpha_size_bytes > 0;
    let alpha_data = match img_file.header.compression_type {
        img_parser::CompressionType::None => img_file.image_data.alpha.to_owned(),
        _ => if has_alpha { decode_lzw2(&img_file.image_data.alpha) } else { vec![0u8; 0] },
    };
    let converted_image = color_data
        .chunks_exact(2)
        .zip(alpha_data.iter().chain(iter::repeat(&255)))
        .map(|(x, y)| (Rgb565::from_rgb565_le([x[0], x[1]]), y))
        .map(|(x, y)| {
            let rgb = x.to_rgb888_components();
            let alpha = if has_alpha { *y } else { 255 };
            [rgb[0], rgb[1], rgb[2], alpha]
        })
        .map(|x| x.map(|y| f32::try_from(y).unwrap() / 255f32))
        .flatten()
        .map(|x| x.to_le_bytes())
        .flatten()
        .collect();
    Image::new(
        Extent3d {
            width: img_file.header.width_px,
            height: img_file.header.height_px,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        converted_image,
        TextureFormat::Rgba32Float,
    )
}

fn ann_sprite_to_image(sprite: &ann_parser::Sprite) -> Image {
    assert_eq!(
        sprite.header.compression_type,
        ann_parser::CompressionType::None
    );
    let converted_image = &sprite.image_data.color;
    let has_alpha = sprite.header.alpha_size_bytes > 0;
    if has_alpha {
        assert_eq!(
            sprite.header.color_size_bytes,
            sprite.header.alpha_size_bytes * 2
        );
    }
    let converted_image = converted_image
        .chunks_exact(2)
        .zip(sprite.image_data.alpha.iter().chain(iter::repeat(&255)))
        .map(|(x, y)| (Rgb565::from_rgb565_le([x[0], x[1]]), y))
        .map(|(x, y)| {
            let rgb = x.to_rgb888_components();
            let alpha = if has_alpha { *y } else { 255 };
            [rgb[0], rgb[1], rgb[2], alpha]
        })
        .map(|x| x.map(|y| f32::try_from(y).unwrap() / 255f32))
        .flatten()
        .map(|x| x.to_le_bytes())
        .flatten()
        .collect();
    Image::new(
        Extent3d {
            width: sprite.header.width_px as u32,
            height: sprite.header.height_px as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        converted_image,
        TextureFormat::Rgba32Float,
    )
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
