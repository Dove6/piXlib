use bevy::{
    ecs::{
        event::EventReader,
        system::{Query, Res, ResMut},
    },
    window::FileDragAndDrop,
};

use crate::resources::{ChosenScene, GamePaths, ScriptRunner};

use super::setup_chooser::{update_scene_list, SceneListComponent};

pub fn handle_dropped_iso(
    mut event_reader: EventReader<FileDragAndDrop>,
    game_paths: Res<GamePaths>,
    mut chosen_scene: ResMut<ChosenScene>,
    mut script_runner: ResMut<ScriptRunner>,
    mut query: Query<&mut SceneListComponent>,
) {
    for event in event_reader.read() {
        println!("{:?}", event);
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            chosen_scene.iso_file_path = Some(path_buf.clone());
            update_scene_list(
                &chosen_scene,
                &game_paths,
                &mut script_runner,
                &mut query.get_single_mut().unwrap(),
            )
        }
    }
}
