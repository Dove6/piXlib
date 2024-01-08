use crate::animation::ann_file_to_animation_bundle;
use crate::arguments::{get_arguments, Arguments};
use crate::image::img_file_to_sprite_bundle;
use crate::iso::{parse_file, read_file_from_iso, read_iso, AmFile};
use crate::resources::{WindowConfiguration, GamePaths};
use bevy::{
    ecs::system::Res,
    prelude::{default, Assets, Camera2dBundle, Commands, Image, ResMut, Transform},
    sprite::TextureAtlas,
};

use std::fs::File;

pub fn setup_viewer(
    window_config: Res<WindowConfiguration>,
    game_paths: Res<GamePaths>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            window_config.size.0 as f32 / 2.0,
            window_config.size.1 as f32 / -2.0,
            0.0,
        ),
        ..default()
    });

    let Arguments {
        path_to_iso,
        path_to_file,
        output_path,
    } = get_arguments().expect("Usage: iso_browser path_to_iso path_to_file_on_iso [output_path]");

    let iso_file = File::open(&path_to_iso).unwrap();
    let mut iso = read_iso(&iso_file);

    let buffer = read_file_from_iso(&mut iso, &path_to_file, output_path.as_deref());

    match parse_file(&buffer, &path_to_file) {
        AmFile::Img(img_file) => {
            commands.spawn(img_file_to_sprite_bundle(&img_file, &mut textures));
        }
        AmFile::Ann(ann_file) => {
            commands.spawn(ann_file_to_animation_bundle(
                &ann_file,
                &mut textures,
                &mut texture_atlases,
            ));
        }
        _ => panic!(),
    };
}
