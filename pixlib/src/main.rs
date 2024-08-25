#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::{
    env,
    sync::{Arc, RwLock},
};

use bevy::{
    ecs::schedule::{OnExit, States},
    math::Vec3,
    prelude::{
        default, App, Camera2dBundle, Commands, DespawnRecursiveExt, PluginGroup, Res, ResMut,
        Startup, Transform,
    },
    render::texture::ImagePlugin,
    window::{PresentMode, Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use bevy_kira_audio::AudioPlugin;
use filesystems::{InsertedDisk, InsertedDiskResource};
use pixlib_parser::{common::IssueManager, runner::ObjectBuilderError};
use plugins::{
    app_plugin::AppPlugin, events_plugin::EventsPlugin, graphics_plugin::GraphicsPlugin,
    inputs_plugin::InputsPlugin, scripts_plugin::ScriptsPlugin, sounds_plugin::SoundsPlugin,
    ui_plugin::UiPlugin,
};
use resources::{ChosenScene, ObjectBuilderIssueManager, RootEntityToDespawn, WindowConfiguration};
use util::IssuePrinter;

const WINDOW_SIZE: (usize, usize) = (800, 600);
const WINDOW_TITLE: &str = "piXlib";

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    SceneChooser,
    SceneViewer,
}

#[allow(clippy::arc_with_non_send_sync)]
fn main() {
    let mut issue_manager: IssueManager<ObjectBuilderError> = Default::default();
    issue_manager.set_handler(Box::new(IssuePrinter));
    let inserted_disk = Arc::new(RwLock::new(
        InsertedDisk::try_from(env::args()).expect("Usage: pixlib path_to_iso [path_to_patch...]"),
    ));
    App::new()
        .add_plugins((
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
            AudioPlugin,
        ))
        .insert_resource(WinitSettings::game())
        .insert_resource(WindowConfiguration {
            size: WINDOW_SIZE,
            title: WINDOW_TITLE,
        })
        .insert_resource(InsertedDiskResource(inserted_disk.clone()))
        .insert_resource(ChosenScene::default())
        .insert_resource(ObjectBuilderIssueManager(issue_manager))
        .init_state::<AppState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnExit(AppState::SceneChooser), cleanup_root)
        .add_systems(OnExit(AppState::SceneViewer), cleanup_root)
        .add_plugins((
            EventsPlugin,
            GraphicsPlugin,
            InputsPlugin,
            ScriptsPlugin {
                inserted_disk,
                window_resolution: WINDOW_SIZE,
            },
            SoundsPlugin,
            AppPlugin,
        ))
        .add_plugins(UiPlugin)
        .run();
}

fn setup_camera(window_config: Res<WindowConfiguration>, mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            window_config.size.0 as f32 / 2.0,
            window_config.size.1 as f32 / 2.0,
            1.0,
        )
        .with_scale(Vec3::new(1f32, -1f32, 1f32)),
        ..default()
    });
}

fn cleanup_root(mut commands: Commands, mut root_entity: ResMut<RootEntityToDespawn>) {
    if let Some(entity) = root_entity.0.take() {
        commands.entity(entity).despawn_recursive();
    }
}

pub mod filesystems;
pub mod plugins;
pub mod resources;
pub mod util;
