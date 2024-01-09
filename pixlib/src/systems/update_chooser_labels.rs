use bevy::{
    ecs::{query::Changed, system::Query},
    hierarchy::Children,
    text::Text,
};

use super::setup_chooser::{ButtonFunctionComponent, SceneListComponent};

pub fn update_chooser_labels(
    scene_list_component_query: Query<
        (&SceneListComponent, &Children),
        Changed<SceneListComponent>,
    >,
    button_query: Query<(&ButtonFunctionComponent, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for (scene_list_component, children) in &scene_list_component_query {
        if scene_list_component.scenes.is_empty() {
            continue;
        }
        for (button_function, button_children) in button_query.iter_many(children) {
            if let ButtonFunctionComponent::Display { offset } = button_function {
                let mut text = text_query.get_mut(button_children[0]).unwrap();
                let displayed_index = scene_list_component.current_index + offset;
                text.sections[0].value = scene_list_component.scenes
                    [displayed_index % scene_list_component.scenes.len()]
                .name
                .clone();
            }
        }
    }
}
