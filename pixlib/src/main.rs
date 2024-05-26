#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod anchors;
pub mod animation;
pub mod arguments;
pub mod components;
pub mod image;
pub mod iso;
pub mod resources;
pub mod states;
pub mod systems;

use std::{env, path::PathBuf, sync::Arc};

use bevy::{
    ecs::{
        change_detection::DetectChanges,
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter, OnExit},
        system::{Res, ResMut},
    },
    prelude::{default, App, PluginGroup, Startup, Update},
    render::texture::ImagePlugin,
    window::{PresentMode, Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use iso::{read_game_definition, read_script};
use pixlib_parser::{classes::CnvType, runner::ScriptSource};
use resources::{
    ChosenScene, DebugSettings, GamePaths, InsertedDisk, SceneDefinition, ScriptRunner,
    WindowConfiguration,
};
use states::AppState;
use systems::{
    animate_sprite, cleanup_root, detect_return_to_chooser, draw_cursor, handle_dropped_iso,
    navigate_chooser, setup, setup_chooser, setup_viewer, update_chooser_labels,
};

const WINDOW_SIZE: (usize, usize) = (800, 600);
const WINDOW_TITLE: &str = "piXlib";

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (WINDOW_SIZE.0 as f32, WINDOW_SIZE.1 as f32).into(),
                        present_mode: PresentMode::AutoVsync,
                        title: WINDOW_TITLE.to_owned(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(WinitSettings::game())
        .insert_resource(WindowConfiguration {
            size: WINDOW_SIZE,
            title: WINDOW_TITLE,
        })
        .insert_resource(DebugSettings {
            force_animation_infinite_looping: true,
        })
        .insert_resource(GamePaths {
            data_directory: "./DANE/".into(),
            game_definition_filename: "./APPLICATION.DEF".into(),
            music_directory: "./".into(),
            dialogues_directory: "./WAVS/".into(),
            sfx_directory: "./WAVS/SFX/".into(),
            common_directory: "./COMMON/".into(),
            classes_directory: "./COMMON/CLASSES/".into(),
        })
        .insert_resource(InsertedDisk::try_from(env::args()).expect("Usage: pixlib path_to_iso"))
        .insert_resource(ChosenScene::default())
        .insert_resource(ScriptRunner::default())
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .add_systems(OnEnter(AppState::SceneChooser), setup_chooser)
        .add_systems(
            Update,
            (handle_dropped_iso, navigate_chooser, update_chooser_labels)
                .run_if(in_state(AppState::SceneChooser)),
        )
        .add_systems(OnExit(AppState::SceneChooser), cleanup_root)
        .add_systems(OnEnter(AppState::SceneViewer), setup_viewer)
        .add_systems(
            Update,
            animate_sprite.run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(Update, reload_main_script)
        .add_systems(
            Update,
            detect_return_to_chooser.run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(OnExit(AppState::SceneViewer), cleanup_root)
        .run();
}

fn reload_main_script(
    inserted_disk: Res<InsertedDisk>,
    game_paths: Res<GamePaths>,
    mut script_runner: ResMut<ScriptRunner>,
    mut chosen_scene: ResMut<ChosenScene>,
) {
    if !inserted_disk.is_changed() {
        return;
    }
    script_runner.unload_all_scripts();
    let Some(iso) = inserted_disk.get() else {
        return;
    };
    let root_script_path = read_game_definition(iso, &game_paths, &mut script_runner);
    let mut vec = Vec::new();
    script_runner
        .0
        .find_objects(|o| matches!(o.content, CnvType::Application(_)), &mut vec);
    if vec.len() != 1 {
        eprintln!(
            "Incorrect number of APPLICATION objects (should be 1): {:?}",
            vec
        );
        return;
    }
    let application_name = vec[0].name.clone();
    let CnvType::Application(application) = &vec[0].content else {
        panic!();
    };
    let application_name = application_name.clone();
    let application_path = application.read().unwrap().path.clone();
    if let Some(application_script_path) = application_path.as_ref() {
        read_script(
            iso,
            application_script_path,
            &application_name,
            &game_paths,
            Some(Arc::clone(&root_script_path)),
            ScriptSource::Application,
            &mut script_runner,
        );
    }
    let CnvType::Application(application) = &vec[0].content else {
        panic!();
    };
    let episode_object_name = if application
        .read()
        .unwrap()
        .episodes
        .as_ref()
        .map(|v| v.len())
        .unwrap_or(0)
        == 1
    {
        application.read().unwrap().episodes.as_ref().unwrap()[0].to_owned()
    } else {
        eprintln!(
            "Unexpected number of episodes (expected 1): {:?}",
            application.read().unwrap().episodes
        );
        return;
    };
    let episode_object_name = episode_object_name.clone();
    if let Some(episode_object) = script_runner.get_object(&episode_object_name) {
        let episode_name = episode_object.name.clone();
        let CnvType::Episode(episode) = &episode_object.content else {
            panic!();
        };
        let episode_name = episode_name.clone();
        let episode_path = episode.read().unwrap().path.clone();
        if let Some(episode_script_path) = episode_path.as_ref() {
            read_script(
                iso,
                episode_script_path,
                &episode_name,
                &game_paths,
                Some(Arc::clone(&root_script_path)),
                ScriptSource::Episode,
                &mut script_runner,
            );
        }
        let CnvType::Episode(episode) = &episode_object.content else {
            unreachable!();
        };
        chosen_scene.list.clear();
        chosen_scene.index = 0;
        for scene_name in episode
            .read()
            .unwrap()
            .scenes
            .as_ref()
            .map(|v| v.iter())
            .unwrap_or(Vec::new().iter())
        {
            if let Some(scene_object) = script_runner.get_object(scene_name) {
                let CnvType::Scene(scene) = &scene_object.content else {
                    panic!();
                };
                let Some(scene_script_path) = scene.read().unwrap().path.clone() else {
                    eprintln!("Scene {} has no path", scene_name);
                    continue;
                };
                let scene_defintion = SceneDefinition {
                    name: scene_name.to_string(),
                    path: PathBuf::from(scene_script_path),
                    background: scene.read().unwrap().background.clone(),
                };
                chosen_scene.list.push(scene_defintion);
            }
        }
        chosen_scene.list.sort();
    } else {
        eprintln!(
            "Could not find episode object with name: {}",
            episode_object_name
        );
    };
}

/*
fn reload_scene_script(
    chosen_scene: Res<ChosenScene>,
    inserted_disk: Res<InsertedDisk>,
    game_paths: Res<GamePaths>,
    mut script_runner: ResMut<ScriptRunner>,
) {
    if !chosen_scene.is_changed() {
        return;
    }
    let Some(iso) = &inserted_disk.get() else {
        return;
    };
    let mut vec = Vec::new();
    script_runner.find_scripts(
        |s| matches!(s.source_kind, ScriptSource::Scene | ScriptSource::CnvLoader),
        &mut vec,
    );
    for episode_script in vec.iter() {
        script_runner.0.unload_script(episode_script);
    }
    let ChosenScene { list, index } = chosen_scene.as_ref();
    let Some(SceneDefinition {
        name,
        path,
        ..
    }) = list.get(*index)
    else {
        println!(
            "Could not load scene script: bad index {} for scene list {:?}",
            index, list
        );
        return;
    };
    read_script(
        iso,
        &path.as_os_str().to_str().unwrap(),
        &name,
        &game_paths,
        script_runner.get_root_script().map(|s| Arc::clone(&s.path)),
        ScriptSource::Scene,
        &mut script_runner,
    );
}
*/
