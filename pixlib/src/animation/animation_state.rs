use bevy::ecs::component::Component;

use super::playback_state::PlaybackState;

#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Copy)]
pub struct AnimationState {
    pub playing_state: PlaybackState,
    pub sequence_idx: usize,
    pub frame_idx: usize,
}
