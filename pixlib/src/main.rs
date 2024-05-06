#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod anchors;
pub mod animation;
pub mod arguments;
pub mod image;
pub mod iso;
pub mod resources;
pub mod states;
pub mod systems;

use std::env;

use bevy::{
    ecs::schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter, OnExit},
    prelude::{default, App, PluginGroup, Startup, Update},
    render::texture::ImagePlugin,
    window::{PresentMode, Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use resources::{
    ChosenScene, DebugSettings, GamePaths, ProgramArguments, ScriptRunner, WindowConfiguration,
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
                .set(ImagePlugin::default_linear()),
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
        .insert_resource(
            ProgramArguments::try_from(env::args()).expect("Usage: pixlib path_to_iso"),
        )
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
            (animate_sprite, detect_return_to_chooser).run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(OnExit(AppState::SceneViewer), cleanup_root)
        .run();
}
