use crate::anchors::{add_tuples, get_anchor, UpdatableAnchor};
use crate::animation::{AnimationDefinition, AnimationState, AnimationTimer};
use crate::animation::{CnvIdentifier, PlaybackState};
use crate::resources::{DebugSettings, ScriptRunner};
use bevy::ecs::system::ResMut;
use bevy::log::{info, warn};
use bevy::{
    ecs::system::Res,
    prelude::Query,
    sprite::{Sprite, TextureAtlas},
    time::Time,
};
use pixlib_formats::file_formats::ann::LoopingSettings;
use pixlib_parser::classes::{Animation, GraphicsEvents, PropertyValue};
use pixlib_parser::runner::{CnvStatement, RunnerContext};

pub fn animate_sprite(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut script_runner: ResMut<ScriptRunner>,
    mut query: Query<(
        &CnvIdentifier,
        &AnimationDefinition,
        &mut AnimationTimer,
        &mut AnimationState,
        &mut Sprite,
        &mut TextureAtlas,
    )>,
) {
    info!("Delta {:?}", time.delta());
    for (ident, animation, mut timer, mut state, mut atlas_sprite, mut atlas) in &mut query {
        let Some(ident) = &ident.0 else {
            continue;
        };
        timer.tick(time.delta());
        let sequence = &animation.sequences[state.sequence_idx];
        let Some(animation_obj_whole) = script_runner.get_object(ident) else {
            warn!(
                "Animation has no associated object in script runner: {}",
                ident
            );
            continue;
        };
        let mut animation_obj_guard = animation_obj_whole.content.write().unwrap();
        let animation_obj = animation_obj_guard
            .as_any_mut()
            .downcast_mut::<Animation>()
            .unwrap();
        info!("{} s events {:?}", ident, animation_obj.events);
        let changed_playback_state = animation_obj
            .events
            .iter()
            .filter(|e| matches!(e, GraphicsEvents::Play(_) | GraphicsEvents::Stop(_)))
            .last()
            .cloned();
        animation_obj
            .events
            .retain(|e| !matches!(e, GraphicsEvents::Play(_) | GraphicsEvents::Stop(_)));
        if sequence.frames.is_empty() {
            if state.playing_state != PlaybackState::Stopped {
                info!("Stopping empty animation {}", ident);
                state.playing_state = PlaybackState::Stopped;
                animation_obj
                    .events
                    .push(GraphicsEvents::Finished(String::new()));
            }
            match changed_playback_state {
                Some(GraphicsEvents::Play(_)) => {
                    info!("Playing empty animation {}", ident);
                    state.frame_idx = 0;
                    state.playing_state = PlaybackState::Forward;
                }
                None => {}
                _ => {
                    info!(
                        "Another event for empty animation {}: {:?}",
                        ident, changed_playback_state
                    );
                }
            }
            let finished_playing = animation_obj
                .events
                .iter()
                .filter(|e| matches!(e, GraphicsEvents::Finished(_)))
                .count()
                > 0;
            animation_obj
                .events
                .retain(|e| !matches!(e, GraphicsEvents::Finished(_)));
            drop(animation_obj_guard);
            if finished_playing {
                let mut context = RunnerContext {
                    self_object: ident.clone(),
                    current_object: ident.clone(),
                };
                if let Some(PropertyValue::Code(handler)) =
                    animation_obj_whole.get_property("ONFINISHED")
                {
                    info!("Calling ONFINISHED on object: {:?}", animation_obj_whole);
                    handler.run(&mut script_runner, &mut context)
                }
            }
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
                    info!(
                        "Stopping animation {} (seq {}, frame {})",
                        ident, state.sequence_idx, state.frame_idx
                    );
                    animation_obj.events.push(GraphicsEvents::Finished(
                        animation
                            .sequences
                            .get(state.sequence_idx)
                            .map(|s| s.name.clone())
                            .or(animation.sequences.first().map(|s| s.name.clone()))
                            .unwrap_or_default(),
                    ));
                }
                let frame = &sequence.frames[state.frame_idx];
                let sprite = &animation.sprites[frame.sprite_idx];
                atlas.index = frame.sprite_idx;
                atlas_sprite.update_anchor(get_anchor(
                    add_tuples(sprite.offset_px, frame.offset_px),
                    sprite.size_px,
                ));
            }
            PlaybackState::Backward if timer.just_finished() => {}
            _ => {}
        }
        match changed_playback_state {
            Some(GraphicsEvents::Play(sequence_name)) => {
                let previous_seq_idx = state.sequence_idx;
                state.sequence_idx = animation
                    .sequences
                    .iter()
                    .position(|s| s.name == *sequence_name)
                    .unwrap_or_default();
                if previous_seq_idx != state.sequence_idx
                    || state.playing_state == PlaybackState::Stopped
                {
                    state.frame_idx = 0;
                }
                state.playing_state = PlaybackState::Forward;
                info!(
                    "Playing animation {} (seq {}, frame {})",
                    ident, state.sequence_idx, state.frame_idx
                );
            }
            Some(GraphicsEvents::Stop(_)) if state.playing_state != PlaybackState::Stopped => {
                info!(
                    "Stopping animation {} (seq {}, frame {})",
                    ident, state.sequence_idx, state.frame_idx
                );
                state.playing_state = PlaybackState::Stopped;
                animation_obj.events.push(GraphicsEvents::Finished(
                    animation
                        .sequences
                        .get(state.sequence_idx)
                        .map(|s| s.name.clone())
                        .or(animation.sequences.first().map(|s| s.name.clone()))
                        .unwrap_or_default(),
                ));
            }
            _ => {}
        }
        info!("{} e events {:?}", ident, animation_obj.events);
        let finished_playing = animation_obj
            .events
            .iter()
            .filter(|e| matches!(e, GraphicsEvents::Finished(_)))
            .count()
            > 0;
        animation_obj
            .events
            .retain(|e| !matches!(e, GraphicsEvents::Finished(_)));
        drop(animation_obj_guard);
        if finished_playing {
            let mut context = RunnerContext {
                self_object: ident.clone(),
                current_object: ident.clone(),
            };
            if let Some(PropertyValue::Code(handler)) =
                animation_obj_whole.get_property("ONFINISHED")
            {
                info!("Calling ONFINISHED on object: {:?}", animation_obj_whole);
                handler.run(&mut script_runner, &mut context)
            }
        }
    }
}
