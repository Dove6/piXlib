use std::path::PathBuf;

use bevy::{
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut},
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
use pixlib_parser::classes::CnvType;

use crate::{
    iso::read_game_definition,
    resources::{ChosenScene, GamePaths, ProgramArguments, RootEntityToDespawn, SceneDefinition},
};

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct SceneListComponent {
    pub scenes: Vec<SceneDefinition>,
    pub current_index: usize,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Copy)]
pub enum ButtonFunctionComponent {
    DecrementIndex,
    IncrementIndex,
    Display { offset: usize },
}

pub fn setup_chooser(
    arguments: Res<ProgramArguments>,
    game_paths: Res<GamePaths>,
    mut commands: Commands,
    mut chosen_scene: ResMut<ChosenScene>,
) {
    chosen_scene.iso_file_path = Some(arguments.path_to_iso.clone());

    let mut scenes = Vec::new();

    if let Some(iso_file_path) = &chosen_scene.iso_file_path {
        let game_definition = read_game_definition(iso_file_path, &game_paths);
        println!("game_definition: {:?}", game_definition);
        for (object_name, cnv_object) in
            game_definition
                .0
                .iter()
                .filter_map(|(k, v)| match &v.content {
                    CnvType::Scene(scene) => Some((k, scene)),
                    _ => None,
                })
        {
            scenes.push(SceneDefinition {
                name: object_name.clone(),
                path: PathBuf::from(&cnv_object.path),
                background: if !cnv_object.background.is_empty() {
                    Some(cnv_object.background.clone())
                } else {
                    None
                },
            });
        }
        scenes.sort();
    }
    println!("scenes: {:?}", scenes);

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
            SceneListComponent {
                scenes,
                current_index: 0,
            },
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
                        "(scene name)",
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
