use crate::animation::ann_file_to_animation_bundle;
use crate::image::img_file_to_sprite_bundle;
use crate::iso::{parse_file, read_file_from_iso, read_script, AmFile};
use crate::resources::{
    ChosenScene, GamePaths, InsertedDisk, RootEntityToDespawn, SceneDefinition, ScriptRunner,
};
use bevy::hierarchy::BuildChildren;
use bevy::log::error;
use bevy::prelude::SpatialBundle;
use bevy::{
    ecs::system::Res,
    prelude::{Assets, Commands, Image, ResMut},
    sprite::TextureAtlasLayout,
};
use pixlib_parser::classes::{CnvObject, CnvType};
use pixlib_parser::runner::{CnvStatement, ScriptSource};

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
    let ChosenScene { list, index } = chosen_scene.as_ref();
    let Some(SceneDefinition { name, path, .. }) = list.get(*index) else {
        println!(
            "Could not load scene script: bad index {} for scene list {:?}",
            index, list
        );
        return;
    };
    let scene_script_path = read_script(
        iso,
        &path.as_os_str().to_str().unwrap(),
        &name,
        &game_paths,
        script_runner.get_root_script().map(|s| Arc::clone(&s.path)),
        ScriptSource::Scene,
        &mut script_runner,
    );
    let Some(scene_object) = script_runner.get_object(&name) else {
        panic!(
            "Could not find scene object {}: {:?}",
            &name,
            script_runner.get_object(&name)
        );
    };
    let CnvObject {
        content: CnvType::Scene(scene_definition),
        ..
    } = scene_object.as_ref()
    else {
        panic!(
            "Could not find scene object {}: {:?}",
            &name,
            script_runner.get_object(&name)
        );
    };
    let Some(scene_path) = scene_definition.read().unwrap().path.clone() else {
        eprintln!("Scene {} has no path", &name);
        return;
    };
    let scene_script = script_runner.get_script(&scene_script_path).unwrap();

    let root_entity = commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            let mut initial_images = vec![];
            if let Some(background_filename) = scene_definition.read().unwrap().background.clone() {
                initial_images.push(OrderedGraphics {
                    file_path: get_path_to_scene_file(
                        &game_paths,
                        &scene_path,
                        &background_filename,
                    )
                    .to_str()
                    .unwrap()
                    .to_owned(),
                    script_index: 0,
                    object_index: 0,
                    priority: 0,
                });
            }
            for OrderedGraphics {
                file_path,
                script_index,
                object_index,
                priority,
            } in initial_images.into_iter().chain(
                scene_script
                    .objects
                    .iter()
                    .filter(|cnv_object| {
                        matches!(
                            cnv_object.content,
                            CnvType::Image(_) | CnvType::Animation(_)
                        )
                    })
                    .map(|cnv_object| {
                        let (filename, priority) = match &cnv_object.content {
                            CnvType::Animation(animation) => {
                                let animation = animation.read().unwrap();
                                (animation.filename.to_owned().unwrap(), animation.priority)
                            }
                            CnvType::Image(image) => {
                                let image = image.read().unwrap();
                                (image.filename.to_owned().unwrap(), image.priority)
                            }
                            _ => panic!(),
                        };
                        OrderedGraphics {
                            file_path: get_path_to_scene_file(&game_paths, &scene_path, &filename)
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

    if let Some(init_beh_obj) = scene_script.get_object("__INIT__") {
        let CnvType::Behavior(init_beh) = &init_beh_obj.content else {
            error!(
                "Expected __INIT__ object to be a behavior, not: {:?}",
                &init_beh_obj.content
            );
            return;
        };
        if let Some(code) = &init_beh.read().unwrap().code {
            code.run(&mut script_runner);
        }
    }
}

fn get_path_to_scene_file(game_paths: &GamePaths, scene_path: &str, filename: &str) -> Arc<Path> {
    let mut path = game_paths.data_directory.join(scene_path).join(filename);
    println!("PATHS: {:?}, {:?}, {:?}", &scene_path, &filename, &path);
    path.as_mut_os_string().make_ascii_uppercase();
    path.into()
}
