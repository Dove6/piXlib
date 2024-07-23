use bevy::{
    ecs::{
        schedule::NextState,
        system::{Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
};

use crate::states::AppState;

pub fn detect_return_to_chooser(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        next_state.set(AppState::SceneChooser);
    }
}
