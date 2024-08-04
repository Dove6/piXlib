use std::fs::File;

use bevy::{
    ecs::{
        event::EventReader,
        system::{Query, ResMut},
    },
    log::info,
    prelude::NonSend,
    window::FileDragAndDrop,
};

use crate::resources::{InsertedDisk, ObjectBuilderIssueManager, ScriptRunner};

use super::setup_chooser::SceneListComponent;

pub fn handle_dropped_iso(
    mut event_reader: EventReader<FileDragAndDrop>,
    mut inserted_disk: ResMut<InsertedDisk>,
    _script_runner: NonSend<ScriptRunner>,
    _issue_manager: ResMut<ObjectBuilderIssueManager>,
    _query: Query<&mut SceneListComponent>,
) {
    for event in event_reader.read() {
        info!("{:?}", event);
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            inserted_disk.insert(File::open(path_buf).unwrap()).unwrap();
        }
    }
}
