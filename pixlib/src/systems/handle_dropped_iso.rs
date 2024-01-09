use bevy::{
    ecs::{
        event::EventReader,
        system::{Query, ResMut},
    },
    window::FileDragAndDrop,
};

use crate::resources::ChosenScene;

use super::setup_chooser::SceneListComponent;

pub fn handle_dropped_iso(
    mut event_reader: EventReader<FileDragAndDrop>,
    mut chosen_scene: ResMut<ChosenScene>,
    mut _query: Query<&mut SceneListComponent>,
) {
    for event in event_reader.read() {
        println!("{:?}", event);
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            chosen_scene.iso_file_path = Some(path_buf.clone());
        }
    }
}
