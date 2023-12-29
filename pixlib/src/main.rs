use bevy::{
    ecs::{component::Component, system::Res},
    math::{UVec2, Vec2},
    prelude::{
        default, App, Assets, Camera, Camera2dBundle, Color, Commands, Deref, DerefMut, Gizmos,
        GlobalTransform, Image, PluginGroup, Query, ResMut, Startup, Transform, Update,
    },
    render::{
        render_resource::{Extent3d, TextureFormat},
        texture::ImagePlugin,
    },
    sprite::{
        Anchor, Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasBuilder,
        TextureAtlasSprite,
    },
    time::{Time, Timer, TimerMode},
    window::{PresentMode, Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use opticaldisc::iso::IsoFs;
use pixlib_formats::{
    compression_algorithms::{lzw2::decode_lzw2, rle::decode_rle},
    file_formats::{
        ann::{self, parse_ann, AnnFile, LoopingSettings},
        arr::{parse_arr, ArrFile},
        img::{parse_img, ImgFile},
        ColorFormat, CompressionType, ImageData,
    },
};

use std::{
    fs::{self, File},
    io::Read,
    iter,
    ops::Add,
};

const WINDOW_SIZE: (f32, f32) = (800., 600.);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WINDOW_SIZE.into(),
                        present_mode: PresentMode::AutoVsync,
                        title: "piXlib".to_owned(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_linear()),
        )
        .insert_resource(WinitSettings::game())
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .add_systems(Update, animate_sprite)
        .run();
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Copy)]
struct AnimationState {
    pub playing_state: PlaybackState,
    pub sequence_idx: usize,
    pub frame_idx: usize,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            playing_state: PlaybackState::Forward,
            sequence_idx: 0,
            frame_idx: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum PlaybackState {
    Forward,
    Backward,
    ForwardPaused,
    BackwardPaused,
    Stopped,
}

#[derive(Component, Clone, Debug, PartialEq, Eq)]
struct AnimationSequenceComponent {
    pub sequences: Vec<AnimationSequence>,
    pub sprites: Vec<SpriteDefinition>,
}

#[derive(Component, Deref, DerefMut, Clone, Debug, PartialEq, Eq)]
struct AnimationTimer(Timer);

impl AnimationSequenceComponent {
    fn new(ann_file: &AnnFile) -> Self {
        let sequences = ann_file
            .sequences
            .iter()
            .map(|sequence: &ann::Sequence| AnimationSequence {
                name: sequence.header.name.0.clone(),
                opacity: sequence.header.opacity,
                looping: sequence.header.looping,
                frames: sequence
                    .frames
                    .iter()
                    .zip(&sequence.header.frame_to_sprite_mapping)
                    .map(|(frame, sprite_idx)| FrameDefinition {
                        name: frame.name.0.clone(),
                        offset_px: (frame.x_position_px.into(), frame.y_position_px.into()),
                        opacity: frame.opacity,
                        sprite_idx: (*sprite_idx).into(),
                    })
                    .collect(),
            })
            .collect();
        let sprites = ann_file
            .sprites
            .iter()
            .map(|sprite| SpriteDefinition {
                name: sprite.header.name.0.clone(),
                size_px: (
                    sprite.header.width_px.into(),
                    sprite.header.height_px.into(),
                ),
                offset_px: (
                    sprite.header.x_position_px.into(),
                    sprite.header.y_position_px.into(),
                ),
            })
            .collect();
        Self { sequences, sprites }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AnimationSequence {
    pub name: String,
    pub opacity: u8,
    pub looping: LoopingSettings,
    pub frames: Vec<FrameDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FrameDefinition {
    pub name: String,
    pub offset_px: (i32, i32),
    pub opacity: u8,
    pub sprite_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SpriteDefinition {
    pub name: String,
    pub size_px: (u32, u32),
    pub offset_px: (i32, i32),
}

fn setup(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(WINDOW_SIZE.0 / 2.0, WINDOW_SIZE.1 / -2.0, 0.0),
        ..default()
    });

    let Arguments {
        path_to_iso,
        path_to_file,
        output_path,
    } = get_arguments();

    let iso_file = File::open(path_to_iso).unwrap();
    let mut iso = read_iso(&iso_file);

    let buffer = read_file_from_iso(&mut iso, &path_to_file, output_path.as_deref());

    match parse_file(&buffer, &path_to_file) {
        AmFile::Img(img_file) => {
            let transform = Transform::from_xyz(
                img_file.header.x_position_px as f32,
                -img_file.header.y_position_px as f32,
                0.0,
            );
            let image = image_data_to_image(
                &img_file.image_data,
                (img_file.header.width_px, img_file.header.height_px),
                img_file.header.color_format,
                img_file.header.compression_type,
                img_file.header.alpha_size_bytes > 0,
            );
            let texture = textures.add(image);

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
        AmFile::Ann(ann_file) => {
            let mut texture_atlas_builder = TextureAtlasBuilder::default()
                .format(TextureFormat::Rgba8UnormSrgb)
                .auto_format_conversion(false)
                .padding(UVec2::new(1, 1))
                .max_size(Vec2::new(16384., 16384.));
            for sprite in ann_file.sprites.iter() {
                let image = image_data_to_image(
                    &sprite.image_data,
                    (
                        sprite.header.width_px.into(),
                        sprite.header.height_px.into(),
                    ),
                    ann_file.header.color_format,
                    sprite.header.compression_type,
                    sprite.header.alpha_size_bytes > 0,
                );
                let texture = textures.add(image);

                texture_atlas_builder
                    .add_texture(texture.id(), textures.get(texture.id()).unwrap());
            }

            let texture_atlas =
                texture_atlases.add(texture_atlas_builder.finish(&mut textures).unwrap());
            let mut sprite = TextureAtlasSprite::default();
            let animation = AnimationSequenceComponent::new(&ann_file);
            let first_non_empty_sequence_idx = animation
                .sequences
                .iter()
                .enumerate()
                .find(|(_, s)| !s.frames.is_empty())
                .map(|(i, _)| i);
            if let Some(first_non_empty_sequence_idx) = first_non_empty_sequence_idx {
                let frame = animation.sequences[first_non_empty_sequence_idx]
                    .frames
                    .first()
                    .unwrap();
                let SpriteDefinition {
                    offset_px, size_px, ..
                } = animation.sprites[frame.sprite_idx];
                sprite.update_anchor(get_anchor(add_tuples(offset_px, frame.offset_px), size_px));
            }

            commands.spawn((
                SpriteSheetBundle {
                    sprite,
                    texture_atlas,
                    ..default()
                },
                animation,
                AnimationState {
                    sequence_idx: first_non_empty_sequence_idx.unwrap_or(0),
                    ..default()
                },
                AnimationTimer(Timer::from_seconds(
                    1.0f32 / ann_file.header.frames_per_second as f32,
                    TimerMode::Repeating,
                )),
            ));
        }
        _ => panic!(),
    };
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

fn offset_by(anchor: Anchor, offset: (f32, f32)) -> Anchor {
    Anchor::Custom(anchor.as_vec() + Vec2::new(-offset.0, offset.1))
}

fn add_tuples<T: Add>(
    a: (T, T),
    b: (T, T),
) -> (<T as std::ops::Add>::Output, <T as std::ops::Add>::Output) {
    (a.0 + b.0, a.1 + b.1)
}

fn get_anchor(offset: (i32, i32), size: (u32, u32)) -> (f32, f32) {
    (
        offset.0 as f32 / size.0 as f32,
        offset.1 as f32 / size.1 as f32,
    )
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationSequenceComponent,
        &mut AnimationTimer,
        &mut AnimationState,
        &mut TextureAtlasSprite,
    )>,
) {
    for (animation, mut timer, mut state, mut atlas_sprite) in &mut query {
        timer.tick(time.delta());
        let sequence = &animation.sequences[state.sequence_idx];
        if sequence.frames.is_empty() {
            return;
        }
        match state.playing_state {
            PlaybackState::Forward if timer.just_finished() => {
                let mut frame_limit = sequence.frames.len();
                if let LoopingSettings::LoopingAfter(looping_after) = sequence.looping {
                    frame_limit = frame_limit.min(looping_after);
                } else {
                    // if state.frame_idx + 1 == frame_limit {
                    //     state.playing_state = PlaybackState::Stopped;
                    //     return;
                    // }
                }
                state.frame_idx = (state.frame_idx + 1) % frame_limit;
                let frame = &sequence.frames[state.frame_idx];
                let sprite = &animation.sprites[frame.sprite_idx];
                atlas_sprite.index = frame.sprite_idx;
                atlas_sprite.update_anchor(get_anchor(
                    add_tuples(sprite.offset_px, frame.offset_px),
                    sprite.size_px,
                ));
            }
            PlaybackState::Backward if timer.just_finished() => return,
            _ => return,
        }
    }
}

trait UpdatableAnchor {
    fn update_anchor(&mut self, offset_from_top_left: (f32, f32));
}

impl UpdatableAnchor for TextureAtlasSprite {
    fn update_anchor(&mut self, offset_from_top_left: (f32, f32)) {
        self.anchor = offset_by(Anchor::TopLeft, offset_from_top_left);
    }
}

struct Arguments {
    path_to_iso: String,
    path_to_file: String,
    output_path: Option<String>,
}

fn get_arguments() -> Arguments {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        panic!("Usage: iso_browser path_to_iso path_to_file_on_iso [output_path]");
    }
    let path_to_iso = args[1].clone();
    let path_to_file = args[2].to_ascii_uppercase().replace('\\', "/");
    let output_path = args.get(3).map(|s| s.to_owned());

    Arguments {
        path_to_iso,
        path_to_file,
        output_path,
    }
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
        CompressionType::Rle => decode_rle(image_data.color, 2),
        CompressionType::Lzw2 => decode_lzw2(image_data.color),
        CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(image_data.color), 2),
        _ => panic!(),
    };
    let alpha_data = match compression_type {
        _ if !has_alpha => vec![],
        CompressionType::None => image_data.alpha.to_owned(),
        CompressionType::Rle => decode_rle(image_data.alpha, 1),
        CompressionType::Lzw2 | CompressionType::Jpeg => decode_lzw2(image_data.alpha),
        CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(image_data.alpha), 1),
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
        .flat_map(|([r, g, b], a)| [r, g, b, *a])
        .collect();
    Image::new(
        Extent3d {
            width: image_size_px.0,
            height: image_size_px.1,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        converted_image,
        TextureFormat::Rgba8UnormSrgb,
    )
}

fn rgb565_to_rgb888(rgb565: [u8; 2]) -> [u8; 3] {
    let rgb565 = u16::from_le_bytes(rgb565);
    let r5 = (rgb565 >> 11) & 0x1f;
    let g6 = (rgb565 >> 5) & 0x3f;
    let b5 = rgb565 & 0x1f;
    [
        (r5 * 255 / 31).try_into().unwrap(),
        (g6 * 255 / 63).try_into().unwrap(),
        (b5 * 255 / 31).try_into().unwrap(),
    ]
}

fn rgb555_to_rgb888(rgb555: [u8; 2]) -> [u8; 3] {
    let rgb555 = u16::from_le_bytes(rgb555);
    let r5 = (rgb555 >> 10) & 0x1f;
    let g5 = (rgb555 >> 5) & 0x1f;
    let b5 = rgb555 & 0x1f;
    [
        (r5 * 255 / 31).try_into().unwrap(),
        (g5 * 255 / 31).try_into().unwrap(),
        (b5 * 255 / 31).try_into().unwrap(),
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

fn read_file_from_iso(
    iso: &mut IsoFs<&File>,
    filename: &str,
    output_filename: Option<&str>,
) -> Vec<u8> {
    let mut buffer = Vec::<u8>::new();
    let bytes_read = iso
        .open_file(filename)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    println!("Read file {} ({} bytes)", filename, bytes_read);

    if let Some(output_path) = output_filename {
        fs::write(output_path, &buffer).expect("Could not write file");
    }

    buffer
}

fn parse_file<'a>(contents: &'a [u8], filename: &str) -> AmFile<'a> {
    let extension = filename
        .split('/')
        .last()
        .unwrap()
        .split('.')
        .last()
        .unwrap();

    match extension {
        "ANN" => AmFile::Ann(parse_ann(contents)),
        "ARR" => AmFile::Arr(parse_arr(contents)),
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
        "IMG" => AmFile::Img(parse_img(contents)),
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

enum AmFile<'a> {
    Ann(AnnFile<'a>),
    Arr(ArrFile),
    Img(ImgFile<'a>),
    None,
}
