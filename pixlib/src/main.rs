#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{
    asset::{AssetMetaCheck, AssetPlugin},
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
use bevy::{
    prelude::AppExtStates,
    state::state::{OnExit, States},
};
use bevy_kira_audio::AudioPlugin;
#[cfg(target_family = "wasm")]
use bevy_web_file_drop::WebFileDropPlugin;
use chrono::Utc;
use filesystems::FileSystemResource;
use plugins::{
    cursor_plugin::CursorPlugin, events_plugin::EventsPlugin, graphics_plugin::GraphicsPlugin,
    inputs_plugin::InputsPlugin, scripts_plugin::ScriptsPlugin, sounds_plugin::SoundsPlugin,
    ui_plugin::UiPlugin,
};
use resources::{ChosenScene, RootEntityToDespawn, WindowConfiguration};

const WINDOW_SIZE: (usize, usize) = (800, 600);
const WINDOW_TITLE: &str = "piXlib";

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    SceneChooser,
    SceneViewer,
}

use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{}  {} - {}",
                Utc::now().format("%+"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

#[allow(clippy::arc_with_non_send_sync)]
fn main() {
    let filesystem_resource = FileSystemResource::default();
    let filesystem = (*filesystem_resource).clone();
    let mut app = App::new();
    #[cfg(target_family = "wasm")]
    app.add_plugins(WebFileDropPlugin);
    app.add_plugins(
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
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(AudioPlugin)
    .insert_resource(WinitSettings::game())
    .insert_resource(WindowConfiguration {
        size: WINDOW_SIZE,
        title: WINDOW_TITLE,
    })
    .insert_resource(filesystem_resource)
    .insert_resource(ChosenScene::default())
    .init_state::<AppState>()
    .add_systems(Startup, setup_camera)
    .add_systems(OnExit(AppState::SceneChooser), cleanup_root)
    .add_systems(OnExit(AppState::SceneViewer), cleanup_root)
    .add_plugins((
        EventsPlugin,
        GraphicsPlugin,
        InputsPlugin,
        ScriptsPlugin {
            filesystem,
            window_resolution: WINDOW_SIZE,
        },
        SoundsPlugin,
        CursorPlugin,
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
