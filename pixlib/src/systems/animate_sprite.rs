use crate::anchors::{add_tuples, get_anchor, UpdatableAnchor};
use crate::animation::PlaybackState;
use crate::animation::{AnimationDefinition, AnimationState, AnimationTimer};
use crate::resources::DebugSettings;
use bevy::{ecs::system::Res, prelude::Query, sprite::TextureAtlasSprite, time::Time};
use pixlib_formats::file_formats::ann::LoopingSettings;

pub fn animate_sprite(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<(
        &AnimationDefinition,
        &mut AnimationTimer,
        &mut AnimationState,
        &mut TextureAtlasSprite,
    )>,
) {
    for (animation, mut timer, mut state, mut atlas_sprite) in &mut query {
        timer.tick(time.delta());
        let sequence = &animation.sequences[state.sequence_idx];
        if sequence.frames.is_empty() {
            return;
        }
        match state.playing_state {
            PlaybackState::Forward if timer.just_finished() => {
                let mut frame_limit = sequence.frames.len();
                if let LoopingSettings::LoopingAfter(looping_after) = sequence.looping {
                    frame_limit = frame_limit.min(looping_after);
                } else if state.frame_idx + 1 == frame_limit
                    && !debug_settings.force_animation_infinite_looping
                {
                    state.playing_state = PlaybackState::Stopped;
                    return;
                }
                state.frame_idx = (state.frame_idx + 1) % frame_limit;
                let frame = &sequence.frames[state.frame_idx];
                let sprite = &animation.sprites[frame.sprite_idx];
                atlas_sprite.index = frame.sprite_idx;
                atlas_sprite.update_anchor(get_anchor(
                    add_tuples(sprite.offset_px, frame.offset_px),
                    sprite.size_px,
                ));
            }
            PlaybackState::Backward if timer.just_finished() => return,
            _ => return,
        }
    }
}
