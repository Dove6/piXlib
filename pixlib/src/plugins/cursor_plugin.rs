use bevy::{
    app::{App, Plugin, Update},
    prelude::{in_state, EventReader, IntoSystemConfigs, OnEnter, Query, With},
    window::{CursorGrabMode, CursorIcon, PrimaryWindow, Window},
};
use pixlib_parser::runner::CursorEvent;

use super::events_plugin::PixlibCursorEvent;
use crate::AppState;

#[derive(Debug, Default)]
pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_cursor.run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(OnEnter(AppState::SceneChooser), reset_cursor);
    }
}

fn update_cursor(
    mut reader: EventReader<PixlibCursorEvent>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();
    for evt in reader.read() {
        match &evt.0 {
            CursorEvent::CursorLocked => window.cursor.grab_mode = CursorGrabMode::Locked,
            CursorEvent::CursorFreed => window.cursor.grab_mode = CursorGrabMode::None,
            CursorEvent::CursorHidden => window.cursor.visible = false,
            CursorEvent::CursorShown => window.cursor.visible = true,
            CursorEvent::CursorSetToPointer => window.cursor.icon = CursorIcon::Pointer,
            CursorEvent::CursorSetToDefault => window.cursor.icon = CursorIcon::Default,
        };
    }
}

fn reset_cursor(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
    window.cursor.icon = CursorIcon::Default;
}
