use crate::anchors::{add_tuples, get_anchor, UpdatableAnchor};
use crate::animation::PlaybackState;
use crate::animation::{AnimationDefinition, AnimationState, AnimationTimer};
use crate::resources::DebugSettings;
use bevy::log::info;
use bevy::{
    ecs::system::Res,
    prelude::Query,
    sprite::{Sprite, TextureAtlas},
    time::Time,
};
use pixlib_formats::file_formats::ann::LoopingSettings;

pub fn animate_sprite(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<(
        &AnimationDefinition,
        &mut AnimationTimer,
        &mut AnimationState,
        &mut Sprite,
        &mut TextureAtlas,
    )>,
) {
    // let mut times: Vec<f32> = Vec::new();
    for (animation, mut timer, mut state, mut atlas_sprite, mut atlas) in &mut query {
        // let t_start = std::time::Instant::now();
        timer.tick(time.delta());
        let sequence = &animation.sequences[state.sequence_idx];
        if sequence.frames.is_empty() {
            continue;
        }
        match state.playing_state {
            PlaybackState::Forward if timer.just_finished() => {
                let mut frame_limit = sequence.frames.len();
                if let LoopingSettings::LoopingAfter(looping_after) = sequence.looping {
                    frame_limit = frame_limit.min(looping_after);
                }
                if matches!(sequence.looping, LoopingSettings::LoopingAfter(_))
                    || debug_settings.force_animation_infinite_looping
                {
                    state.frame_idx =
                        (state.frame_idx + timer.times_finished_this_tick() as usize) % frame_limit;
                } else {
                    state.frame_idx = frame_limit.saturating_sub(1);
                    state.playing_state = PlaybackState::Stopped;
                }
                let frame = &sequence.frames[state.frame_idx];
                let sprite = &animation.sprites[frame.sprite_idx];
                atlas.index = frame.sprite_idx;
                atlas_sprite.update_anchor(get_anchor(
                    add_tuples(sprite.offset_px, frame.offset_px),
                    sprite.size_px,
                ));
            }
            PlaybackState::Backward if timer.just_finished() => continue,
            _ => continue,
        }
        // times.push((std::time::Instant::now() - t_start).as_secs_f32());
    }
    // println!("Times: {:?}", times);
}
