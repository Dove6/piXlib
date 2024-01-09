use crate::animation::ann_file_to_animation_bundle;
use crate::image::img_file_to_sprite_bundle;
use crate::iso::{parse_file, read_file_from_iso, read_iso, AmFile};
use crate::resources::{ChosenScene, GamePaths, RootEntityToDespawn};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;
use bevy::{
    ecs::system::Res,
    prelude::{Assets, Commands, Image, ResMut},
    sprite::TextureAtlas,
};

use std::fs::File;
use std::iter;
use std::path::PathBuf;

pub fn setup_viewer(
    game_paths: Res<GamePaths>,
    chosen_scene: Res<ChosenScene>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Image>>,
) {
    let ChosenScene {
        iso_file_path: Some(iso_file_path),
        scene_definition: Some(scene_definition),
    } = chosen_scene.as_ref()
    else {
        eprintln!("ChosenScene resource: {:?}", chosen_scene);
        panic!("Expected fields of the ChosenScene resource to have a value");
    };
    let scene_path = game_paths
        .data_directory
        .join(scene_definition.path.to_str().unwrap().replace('\\', "/"));

    let iso_file = File::open(&iso_file_path).unwrap();
    let mut iso = read_iso(&iso_file);

    let file_path_inside_iso =
        get_path_to_scene_file(&scene_path, &(scene_definition.name.clone() + ".CNV"));
    let buffer = read_file_from_iso(&mut iso, &file_path_inside_iso, None);

    let root_entity = commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            let mut initial_images = vec![];
            if let Some(background_filename) = scene_definition.background.as_ref() {
                initial_images.push(get_path_to_scene_file(&scene_path, background_filename));
            }
            match parse_file(&buffer, &file_path_inside_iso) {
                AmFile::Cnv(cnv_file) => {
                    for image_path in initial_images.into_iter().chain(cnv_file
                        .0
                        .iter()
                        .filter(|(_, parameters)| {
                            parameters.contains_key("TYPE")
                                && parameters["TYPE"] == "IMAGE"
                                && parameters.contains_key("FILENAME")
                        })
                        .map(|(_, parameters)| {
                            get_path_to_scene_file(
                                &scene_path,
                                parameters.get("FILENAME").as_ref().unwrap(),
                            )
                        }))
                    {
                        println!("Handling image file: {image_path}");
                        let buffer = read_file_from_iso(&mut iso, &image_path, None);
                        match parse_file(&buffer, &image_path) {
                            AmFile::Img(img_file) => {
                                parent.spawn(img_file_to_sprite_bundle(&img_file, &mut textures));
                            }
                            _ => panic!(),
                        }
                    }
                    for animation_path in cnv_file
                        .0
                        .iter()
                        .filter(|(_, parameters)| {
                            parameters.contains_key("TYPE")
                                && parameters["TYPE"] == "ANIMO"
                                && parameters.contains_key("FILENAME")
                        })
                        .map(|(_, parameters)| {
                            get_path_to_scene_file(
                                &scene_path,
                                parameters.get("FILENAME").as_ref().unwrap(),
                            )
                        })
                    {
                        println!("Handling animation file: {animation_path}");
                        let buffer = read_file_from_iso(&mut iso, &animation_path, None);
                        match parse_file(&buffer, &animation_path) {
                            AmFile::Ann(ann_file) => {
                                parent.spawn(ann_file_to_animation_bundle(
                                    &ann_file,
                                    &mut textures,
                                    &mut texture_atlases,
                                ));
                            }
                            _ => panic!(),
                        }
                    }
                }
                _ => panic!(),
            };
        })
        .id();
    commands.insert_resource(RootEntityToDespawn(Some(root_entity)));
}

fn get_path_to_scene_file(scene_path: &PathBuf, filename: &str) -> String {
    scene_path.join(filename).to_str().unwrap().to_owned()
}
