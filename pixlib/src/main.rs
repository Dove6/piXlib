pub mod anchors;
pub mod animation;
pub mod arguments;
pub mod image;
pub mod iso;
pub mod resources;
pub mod systems;

use bevy::{
    prelude::{default, App, PluginGroup, Startup, Update},
    render::texture::ImagePlugin,
    window::{PresentMode, Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use resources::{DebugSettings, WindowConfiguration};
use systems::{animate_sprite, draw_cursor, setup};

const WINDOW_SIZE: (usize, usize) = (800, 600);
const WINDOW_TITLE: &'static str = "piXlib";

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
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .add_systems(Update, animate_sprite)
        .run();
}
