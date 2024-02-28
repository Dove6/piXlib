use crate::animation::ann_file_to_animation_bundle;
use crate::image::img_file_to_sprite_bundle;
use crate::iso::{parse_file, read_file_from_iso, read_iso, AmFile, CnvType};
use crate::resources::{ChosenScene, GamePaths, RootEntityToDespawn};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;
use bevy::{
    ecs::system::Res,
    prelude::{Assets, Commands, Image, ResMut},
    sprite::TextureAtlasLayout,
};

use std::fs::File;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct OrderedGraphics {
    pub file_path: String,
    pub script_index: usize,
    pub object_index: usize,
    pub priority: i32,
}

pub fn setup_viewer(
    game_paths: Res<GamePaths>,
    chosen_scene: Res<ChosenScene>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
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

    let iso_file = File::open(iso_file_path).unwrap();
    let mut iso = read_iso(&iso_file);

    let file_path_inside_iso =
        get_path_to_scene_file(&scene_path, &(scene_definition.name.clone() + ".CNV"));
    let buffer = read_file_from_iso(&mut iso, &file_path_inside_iso, None);

    let root_entity = commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            let mut initial_images = vec![];
            if let Some(background_filename) = scene_definition.background.as_ref() {
                initial_images.push(OrderedGraphics {
                    file_path: get_path_to_scene_file(&scene_path, background_filename),
                    script_index: 0,
                    object_index: 0,
                    priority: 0,
                });
            }
            let AmFile::Cnv(cnv_file) = parse_file(&buffer, &file_path_inside_iso) else {
                panic!();
            };
            for OrderedGraphics {
                file_path,
                script_index,
                object_index,
                priority,
            } in initial_images.into_iter().chain(
                cnv_file
                    .0
                    .iter()
                    .filter(|(_, cnv_object)| {
                        matches!(
                            cnv_object.r#type,
                            Some(CnvType::Image) | Some(CnvType::Animation)
                        ) && cnv_object.properties.contains_key("FILENAME")
                    })
                    .map(|(_, cnv_object)| OrderedGraphics {
                        file_path: get_path_to_scene_file(
                            &scene_path,
                            cnv_object.properties.get("FILENAME").as_ref().unwrap(),
                        ),
                        script_index: 1,
                        object_index: cnv_object.index.unwrap_or(0),
                        priority: cnv_object
                            .properties
                            .get("PRIORITY")
                            .and_then(|priority| Some(priority.parse::<i32>().unwrap_or(0)))
                            .unwrap_or(0),
                    }),
            ) {
                let buffer = read_file_from_iso(&mut iso, &file_path, None);
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

fn get_path_to_scene_file(scene_path: &Path, filename: &str) -> String {
    scene_path.join(filename).to_str().unwrap().to_owned()
}
