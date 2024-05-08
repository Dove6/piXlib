use bevy::{
    ecs::{
        change_detection::DetectChanges,
        system::{Query, Res},
    },
    hierarchy::Children,
    text::Text,
};

use crate::resources::ChosenScene;

use super::setup_chooser::{ButtonFunctionComponent, SceneListComponent};

pub fn update_chooser_labels(
    chosen_scene: Res<ChosenScene>,
    scene_list_component_query: Query<(&SceneListComponent, &Children)>,
    button_query: Query<(&ButtonFunctionComponent, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    if !chosen_scene.is_changed() || chosen_scene.list.is_empty() {
        return;
    }
    for (_, children) in &scene_list_component_query {
        for (button_function, button_children) in button_query.iter_many(children) {
            if let ButtonFunctionComponent::Display { offset } = button_function {
                let mut text = text_query.get_mut(button_children[0]).unwrap();
                let displayed_index = chosen_scene.index + offset;
                text.sections[0].value = chosen_scene.list
                    [displayed_index % chosen_scene.list.len()]
                .name
                .clone();
            }
        }
    }
}
