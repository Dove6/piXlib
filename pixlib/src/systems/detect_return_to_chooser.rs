use bevy::{
    ecs::{
        schedule::NextState,
        system::{Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    log::{info, warn},
};
use pixlib_parser::classes::{Episode, EpisodeEvents};

use crate::{resources::ScriptRunner, states::AppState};

pub fn detect_return_to_chooser(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        next_state.set(AppState::SceneChooser);
    }
}

pub fn detect_return_to_chooser_goto(
    script_runner: Res<ScriptRunner>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut episodes = Vec::new();
    script_runner.find_objects(
        |o| {
            o.content
                .read()
                .unwrap()
                .as_any()
                .downcast_ref::<Episode>()
                .is_some()
        },
        &mut episodes,
    );
    let Some(episode_obj) = episodes.iter().next() else {
        warn!("Could not find EPISODE object",);
        return;
    };
    let mut episode_guard = episode_obj.content.write().unwrap();
    let Some(episode) = episode_guard.as_any_mut().downcast_mut::<Episode>() else {
        return;
    };
    let changed_scene = episode
        .events
        .iter()
        .filter(|e| matches!(e, EpisodeEvents::GoTo(_)))
        .next();
    let changed_scene = changed_scene.map(|e| e.clone());
    episode
        .events
        .retain(|e| !matches!(e, EpisodeEvents::GoTo(_)));
    if let Some(EpisodeEvents::GoTo(next_scene)) = changed_scene { // TODO: switch scene
        info!("EpisodeEvents::GoTo({})", next_scene);
        next_state.set(AppState::SceneChooser);
    }
}
