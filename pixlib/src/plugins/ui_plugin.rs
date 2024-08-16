use std::fs::File;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        query::{Changed, With},
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Children, Parent},
    input::ButtonInput,
    log::info,
    prelude::{default, in_state, EventReader, IntoSystemConfigs, KeyCode, OnEnter},
    render::color::Color,
    text::{Text, TextStyle},
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        widget::Button,
        AlignItems, BackgroundColor, BorderColor, Interaction, JustifyContent, Style, UiRect, Val,
    },
    window::FileDragAndDrop,
};

use crate::{
    filesystems::InsertedDiskResource,
    resources::{ChosenScene, RootEntityToDespawn},
    AppState,
};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Debug, Default)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_dropped_iso, navigate_chooser, update_chooser_labels)
                .run_if(in_state(AppState::SceneChooser)),
        )
        // .add_systems(Update, draw_cursor)
        .add_systems(OnEnter(AppState::SceneChooser), setup_chooser)
        .add_systems(
            Update,
            (detect_return_to_chooser).run_if(in_state(AppState::SceneViewer)),
        );
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct SceneListComponent {
    pub current_index: usize,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Copy)]
pub enum ButtonFunctionComponent {
    DecrementIndex,
    IncrementIndex,
    Display { offset: usize },
}

pub fn setup_chooser(chosen_scene: Res<ChosenScene>, mut commands: Commands) {
    let scene_list = SceneListComponent::default();

    let root_entity = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            scene_list,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        ..default()
                    },
                    ButtonFunctionComponent::DecrementIndex,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "<",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(300.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        ..default()
                    },
                    ButtonFunctionComponent::Display { offset: 0 },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        chosen_scene
                            .list
                            .get(chosen_scene.index)
                            .map(|s| s.name.clone())
                            .unwrap_or("(Empty list)".to_owned()),
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        ..default()
                    },
                    ButtonFunctionComponent::IncrementIndex,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        ">",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .id();
    commands.insert_resource(RootEntityToDespawn(Some(root_entity)));
}

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

pub fn handle_dropped_iso(
    mut event_reader: EventReader<FileDragAndDrop>,
    inserted_disk: Res<InsertedDiskResource>,
) {
    for event in event_reader.read() {
        info!("Drag and drop event: {:?}", event);
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let mut guard = inserted_disk.0.as_ref().write().unwrap();
            guard.insert(File::open(path_buf).unwrap()).unwrap();
        }
    }
}

pub fn detect_return_to_chooser(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        next_state.set(AppState::SceneChooser);
    }
}
