use bevy::{
    ecs::{
        query::{Changed, With},
        schedule::NextState,
        system::{Query, ResMut},
    },
    hierarchy::{Children, Parent},
    render::color::Color,
    ui::{widget::Button, BackgroundColor, BorderColor, Interaction},
};

use crate::{resources::ChosenScene, states::AppState};

use super::setup_chooser::{ButtonFunctionComponent, SceneListComponent};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub fn navigate_chooser(
    mut interaction_query: Query<
        (
            &Interaction,
            &ButtonFunctionComponent,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &Parent,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut scene_list_component_query: Query<&mut SceneListComponent>,
    mut chosen_scene: ResMut<ChosenScene>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button_function, mut color, mut border_color, children, parent) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                let scene_list_component = scene_list_component_query.get(parent.get()).unwrap();
                if scene_list_component.scenes.len() > 0 {
                    if matches!(
                        *button_function,
                        ButtonFunctionComponent::IncrementIndex
                            | ButtonFunctionComponent::DecrementIndex,
                    ) {
                        let mut scene_list_component =
                            scene_list_component_query.get_mut(parent.get()).unwrap();
                        if *button_function == ButtonFunctionComponent::IncrementIndex {
                            scene_list_component.current_index =
                                (scene_list_component.current_index + 1)
                                    % scene_list_component.scenes.len();
                        } else if *button_function == ButtonFunctionComponent::DecrementIndex {
                            scene_list_component.current_index = (scene_list_component
                                .current_index
                                + scene_list_component.scenes.len()
                                - 1)
                                % scene_list_component.scenes.len();
                        }
                    } else if let ButtonFunctionComponent::Display { offset } = button_function {
                        let displayed_index = scene_list_component.current_index + offset;
                        chosen_scene.scene_definition = Some(
                            scene_list_component.scenes
                                [displayed_index % scene_list_component.scenes.len()]
                            .clone(),
                        );
                        next_state.set(AppState::SceneViewer);
                    }
                }
            }
        }
    }
}
