use bevy::{
    app::{App, Plugin, Update},
    prelude::{in_state, EventReader, IntoSystemConfigs, OnEnter, Query, With},
    window::{CursorGrabMode, CursorIcon, PrimaryWindow, Window},
};

use super::events_plugin::PixlibApplicationEvent;
use crate::AppState;

#[derive(Debug, Default)]
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_cursor.run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(OnEnter(AppState::SceneChooser), reset_cursor);
    }
}

fn update_cursor(
    mut reader: EventReader<PixlibApplicationEvent>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();
    for evt in reader.read() {
        match &evt.0 {
            pixlib_parser::runner::ApplicationEvent::CursorLocked => {
                window.cursor.grab_mode = CursorGrabMode::Locked
            }
            pixlib_parser::runner::ApplicationEvent::CursorFreed => {
                window.cursor.grab_mode = CursorGrabMode::None
            }
            pixlib_parser::runner::ApplicationEvent::CursorHidden => window.cursor.visible = false,
            pixlib_parser::runner::ApplicationEvent::CursorShown => window.cursor.visible = true,
            pixlib_parser::runner::ApplicationEvent::CursorSetToPointer => {
                window.cursor.icon = CursorIcon::Pointer
            }
            pixlib_parser::runner::ApplicationEvent::CursorSetToDefault => {
                window.cursor.icon = CursorIcon::Default
            }
            _ => {}
        };
    }
}

fn reset_cursor(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
    window.cursor.icon = CursorIcon::Default;
}
