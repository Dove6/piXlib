use crate::animation::ann_file_to_animation_bundle;
use crate::image::img_file_to_sprite_bundle;
use crate::iso::{parse_file, read_file_from_iso, AmFile};
use crate::resources::{ChosenScene, GamePaths, InsertedDisk, RootEntityToDespawn, ScriptRunner};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;
use bevy::{
    ecs::system::Res,
    prelude::{Assets, Commands, Image, ResMut},
    sprite::TextureAtlasLayout,
};
use pixlib_parser::classes::{self, CnvType};
use pixlib_parser::common::Position;
use pixlib_parser::runner::ScriptSource;

use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct OrderedGraphics {
    pub file_path: String,
    pub script_index: usize,
    pub object_index: usize,
    pub priority: i32,
}

pub fn setup_viewer(
    game_paths: Res<GamePaths>,
    inserted_disk: Res<InsertedDisk>,
    chosen_scene: Res<ChosenScene>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut textures: ResMut<Assets<Image>>,
    mut script_runner: ResMut<ScriptRunner>,
) {
    let Some(iso) = inserted_disk.get() else {
        panic!("No disk inserted!");
    };
    let ChosenScene {
        scene_definition: Some(scene_definition),
    } = chosen_scene.as_ref()
    else {
        panic!("No scene chosen!");
    };
    let scene_path = game_paths
        .data_directory
        .join(scene_definition.path.to_str().unwrap().replace('\\', "/"));

    let file_path_inside_iso = get_path_to_scene_file(
        &scene_path,
        &PathBuf::from(scene_definition.name.clone() + ".CNV"),
    );
    let buffer = read_file_from_iso(iso, &file_path_inside_iso, None);

    let root_entity = commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            let mut initial_images = vec![];
            if let Some(background_filename) = scene_definition.background.as_ref() {
                initial_images.push(OrderedGraphics {
                    file_path: get_path_to_scene_file(
                        &scene_path,
                        &PathBuf::from(background_filename),
                    )
                    .to_str()
                    .unwrap()
                    .to_owned(),
                    script_index: 0,
                    object_index: 0,
                    priority: 0,
                });
            }
            let AmFile::Cnv(cnv_file) = parse_file(&buffer, file_path_inside_iso.to_str().unwrap())
            else {
                panic!();
            };
            if let Err(parsing_err) = script_runner.0.load_script(
                Arc::clone(&file_path_inside_iso),
                cnv_file.0.char_indices().map(|(i, c)| {
                    Ok((
                        Position {
                            line: 1,
                            column: 1 + i,
                            character: i,
                        },
                        c,
                        Position {
                            line: 1,
                            column: 2 + i,
                            character: i + 1,
                        },
                    ))
                }),
                None,
                ScriptSource::Application,
            ) {
                panic!(
                    "Error loading script {:?}: {}",
                    &file_path_inside_iso, parsing_err
                );
            }
            let scene_definition = script_runner.0.get_script(&file_path_inside_iso).unwrap();
            for OrderedGraphics {
                file_path,
                script_index,
                object_index,
                priority,
            } in initial_images.into_iter().chain(
                scene_definition
                    .objects
                    .iter()
                    .filter(|(_, cnv_object)| {
                        matches!(
                            cnv_object.content,
                            CnvType::Image(_) | CnvType::Animation(_)
                        )
                    })
                    .map(|(_, cnv_object)| {
                        let (filename, priority) = match &cnv_object.content {
                            CnvType::Animation(classes::Animation {
                                filename: Some(filename),
                                priority,
                                ..
                            }) => (filename, *priority),
                            CnvType::Image(classes::Image {
                                filename: Some(filename),
                                priority,
                                ..
                            }) => (filename, *priority),
                            _ => panic!(),
                        };
                        OrderedGraphics {
                            file_path: get_path_to_scene_file(
                                &scene_path,
                                &PathBuf::from(filename),
                            )
                            .to_str()
                            .unwrap()
                            .to_owned(),
                            script_index: 1,
                            object_index: cnv_object.index,
                            priority: priority.unwrap_or(0),
                        }
                    }),
            ) {
                let buffer = read_file_from_iso(iso, &PathBuf::from(&file_path), None);
                let z_position =
                    priority as f32 + (script_index * 1000 + object_index) as f32 / 100000f32;
                match parse_file(&buffer, &file_path) {
                    AmFile::Img(img_file) => {
                        let mut bundle = img_file_to_sprite_bundle(&img_file, &mut textures);
                        bundle.transform.translation.z = z_position;
                        println!(
                            "Handling image file: {file_path} z: {}",
                            &bundle.transform.translation.z
                        );
                        parent.spawn(bundle);
                    }
                    AmFile::Ann(ann_file) => {
                        let mut bundle = ann_file_to_animation_bundle(
                            &ann_file,
                            &mut textures,
                            &mut texture_atlases,
                        );
                        bundle.sprite_sheet.transform.translation.z = z_position;
                        println!(
                            "Handling animation file: {file_path} z: {}",
                            &bundle.sprite_sheet.transform.translation.z
                        );
                        parent.spawn(bundle);
                    }
                    _ => panic!(),
                }
            }
        })
        .id();
    commands.insert_resource(RootEntityToDespawn(Some(root_entity)));
}

fn get_path_to_scene_file(scene_path: &Path, filename: &Path) -> Arc<Path> {
    let mut path = scene_path.join(filename);
    println!("PATHS: {:?}, {:?}, {:?}", &scene_path, &filename, &path,);
    path.as_mut_os_string().make_ascii_uppercase();
    path.into()
}
