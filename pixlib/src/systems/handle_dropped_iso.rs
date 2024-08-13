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

use crate::resources::{InsertedDiskResource, ObjectBuilderIssueManager, ScriptRunner};

use super::setup_chooser::SceneListComponent;

pub fn handle_dropped_iso(
    mut event_reader: EventReader<FileDragAndDrop>,
    inserted_disk: NonSend<InsertedDiskResource>,
    _script_runner: NonSend<ScriptRunner>,
    _issue_manager: ResMut<ObjectBuilderIssueManager>,
    _query: Query<&mut SceneListComponent>,
) {
    for event in event_reader.read() {
        info!("{:?}", event);
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let mut guard = inserted_disk.0.as_ref().borrow_mut();
            guard.insert(File::open(path_buf).unwrap()).unwrap();
        }
    }
}
