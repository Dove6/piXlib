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

use crate::{
    iso::read_game_definition,
    resources::{ChosenScene, GamePaths, ProgramArguments, RootEntityToDespawn},
};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneDefinition {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct SceneListComponent(pub Vec<SceneDefinition>);

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
        for (object_name, properties) in game_definition.0.iter() {
            if !properties.contains_key("TYPE")
                || properties["TYPE"].to_uppercase() != "SCENE"
                || !properties.contains_key("PATH")
            {
                continue;
            }
            scenes.push(SceneDefinition {
                name: object_name.clone(),
                path: properties["PATH"].replace('\\', "/").into(),
            });
        }
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
            SceneListComponent(scenes),
        ))
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
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
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .id();
    commands.insert_resource(RootEntityToDespawn(root_entity));
}
