use bevy::ecs::component::Component;

use super::playback_state::PlaybackState;

#[derive(Component, Clone, Debug, PartialEq, Eq, Copy)]
pub struct AnimationState {
    pub playing_state: PlaybackState,
    pub sequence_idx: usize,
    pub frame_idx: usize,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            playing_state: PlaybackState::Forward,
            sequence_idx: 0,
            frame_idx: 0,
        }
    }
}
