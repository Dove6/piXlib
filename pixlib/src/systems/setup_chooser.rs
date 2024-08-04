use bevy::{
    ecs::{
        component::Component,
        system::{Commands, Res},
    },
    hierarchy::BuildChildren,
    prelude::default,
    render::color::Color,
    text::TextStyle,
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        AlignItems, BorderColor, JustifyContent, Style, UiRect, Val,
    },
};

use crate::resources::{ChosenScene, RootEntityToDespawn};

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
