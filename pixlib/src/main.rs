mod arr_parser;
mod img_parser;

use arr_parser::ArrFile;
use bevy::{
    prelude::{
        default, App, Assets, BuildChildren, Camera2dBundle, Color, Commands, Handle, Image,
        NodeBundle, PluginGroup, ResMut, Startup,
    },
    render::render_resource::{Extent3d, TextureFormat},
    ui::{
        AlignItems, FlexDirection, JustifyContent, RelativeCursorPosition, Style, UiImage, UiRect,
        Val,
    },
    window::{Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use img_parser::ImgFile;
use opticaldisc::iso::IsoFs;
use rgb565::Rgb565;

use crate::arr_parser::parse_arr;
use crate::img_parser::parse_img;
use std::{
    fs::{self, File},
    io::Read,
    iter,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .run();
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

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let parsed_file = parse_file();

    if let AmFile::Img(img_file) = parsed_file {
        let image = to_image(&img_file);
        let handle = images.add(image);
        draw_ui(
            commands,
            (
                img_file.header.width_px as usize,
                img_file.header.height_px as usize,
            ),
            handle,
        );
    } else {
        panic!();
    }
}

fn draw_ui(mut commands: Commands, image_size: (usize, usize), image: Handle<Image>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(image_size.0 as f32),
                        height: Val::Px(image_size.1 as f32),
                        ..default()
                    },
                    // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                UiImage::new(image),
            ));
        });
}

fn to_image(img_file: &ImgFile) -> Image {
    let converted_image = &img_file.image_data.color;
    let has_alpha = img_file.header.alpha_size_bytes > 0;
    if has_alpha {
        assert_eq!(
            img_file.header.color_size_bytes,
            img_file.header.alpha_size_bytes * 2
        );
    }
    let converted_image = converted_image
        .chunks_exact(2)
        .zip(img_file.image_data.alpha.iter().chain(iter::repeat(&255)))
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
        "ANN" => {
            println!("Detected animation file.");
            AmFile::None
        }
        "ARR" => AmFile::Arr(parse_arr(&buffer)),
        "CLASS" | "CNV" | "DEF" => {
            println!("Detected script file.");
            AmFile::None
        }
        "DTA" => {
            parse_dta(&buffer);
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
            parse_seq(&buffer);
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

fn parse_dta(data: &Vec<u8>) {
    println!("Detected text database file.");
}

fn parse_seq(data: &Vec<u8>) {
    println!("Detected animation sequence description file.");
}

enum AmFile {
    Arr(ArrFile),
    Img(ImgFile),
    None,
}
