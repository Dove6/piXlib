use bevy::{
    ecs::{
        query::{Changed, With},
        schedule::NextState,
        system::{Query, ResMut},
    },
    hierarchy::Parent,
    render::color::Color,
    ui::{widget::Button, BackgroundColor, BorderColor, Interaction},
};

use crate::{resources::ChosenScene, states::AppState};

use super::setup_chooser::ButtonFunctionComponent;

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
            &Parent,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut chosen_scene: ResMut<ChosenScene>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button_function, mut color, mut border_color, _) in &mut interaction_query {
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
                if chosen_scene.list.is_empty() {
                    return;
                }
                if matches!(
                    *button_function,
                    ButtonFunctionComponent::IncrementIndex
                        | ButtonFunctionComponent::DecrementIndex,
                ) {
                    if *button_function == ButtonFunctionComponent::IncrementIndex {
                        chosen_scene.index = (chosen_scene.index + 1) % chosen_scene.list.len();
                    } else if *button_function == ButtonFunctionComponent::DecrementIndex {
                        chosen_scene.index = (chosen_scene.index + chosen_scene.list.len() - 1)
                            % chosen_scene.list.len();
                    }
                } else if let ButtonFunctionComponent::Display { offset } = button_function {
                    let displayed_index = chosen_scene.index + offset;
                    chosen_scene.index = displayed_index % chosen_scene.list.len();
                    next_state.set(AppState::SceneViewer);
                }
            }
        }
    }
}
