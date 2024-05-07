use std::fs::File;

use bevy::{
    ecs::{
        event::EventReader,
        system::{Query, Res, ResMut},
    },
    window::FileDragAndDrop,
};

use crate::resources::{ChosenScene, GamePaths, InsertedDisk, ScriptRunner};

use super::setup_chooser::{update_scene_list, SceneListComponent};

pub fn handle_dropped_iso(
    mut event_reader: EventReader<FileDragAndDrop>,
    game_paths: Res<GamePaths>,
    mut inserted_disk: ResMut<InsertedDisk>,
    mut chosen_scene: ResMut<ChosenScene>,
    mut script_runner: ResMut<ScriptRunner>,
    mut query: Query<&mut SceneListComponent>,
) {
    for event in event_reader.read() {
        println!("{:?}", event);
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            chosen_scene.scene_definition = None;
            inserted_disk.insert(File::open(path_buf).unwrap()).unwrap();
            update_scene_list(
                &inserted_disk,
                &game_paths,
                &mut script_runner,
                &mut query.get_single_mut().unwrap(),
            )
        }
    }
}
