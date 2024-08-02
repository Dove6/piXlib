use std::path::PathBuf;

use bevy::{
    ecs::{
        component::Component,
        system::{Commands, Res},
    },
    hierarchy::BuildChildren,
    log::info,
    prelude::default,
    render::color::Color,
    text::TextStyle,
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        AlignItems, BorderColor, JustifyContent, Style, UiRect, Val,
    },
};
use pixlib_parser::classes::PropertyValue;

use crate::{
    iso::read_game_definition,
    resources::{
        ChosenScene, GamePaths, InsertedDisk, ObjectBuilderIssueManager, RootEntityToDespawn,
        SceneDefinition, ScriptRunner,
    },
};

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
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

pub fn setup_chooser(chosen_scene: Res<ChosenScene>, mut commands: Commands) {
    let scene_list = SceneListComponent::default();
    // update_scene_list(
    //     &inserted_disk,
    //     &game_paths,
    //     &mut script_runner,
    //     &mut scene_list,
    // );

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

pub fn update_scene_list(
    inserted_disk: &InsertedDisk,
    game_paths: &GamePaths,
    script_runner: &mut ScriptRunner,
    scene_list: &mut SceneListComponent,
    issue_manager: &mut ObjectBuilderIssueManager,
) {
    scene_list.scenes.clear();
    if let Some(iso) = inserted_disk.get() {
        let game_definition_path =
            read_game_definition(iso, game_paths, script_runner, issue_manager);
        let game_definition = script_runner
            .0
            .read()
            .unwrap()
            .get_script(&game_definition_path)
            .unwrap();
        info!("game_definition: {:?}", game_definition);
        for (name, path, background) in
            game_definition
                .read()
                .unwrap()
                .objects
                .iter()
                .filter_map(|o| {
                    let content_guard = o.content.read().unwrap();
                    let content = content_guard.as_ref().unwrap();
                    if content.get_type_id() == "SCENE" {
                        Some((
                            o.name.clone(),
                            content.get_property("PATH").and_then(|v| match v {
                                PropertyValue::String(s) => Some(s),
                                _ => None,
                            }),
                            content.get_property("BACKGROUND").and_then(|v| match v {
                                PropertyValue::String(s) => Some(s),
                                _ => None,
                            }),
                        ))
                    } else {
                        None
                    }
                })
        {
            scene_list.scenes.push(SceneDefinition {
                name,
                path: PathBuf::from(path.unwrap()),
                background,
            });
        }
        scene_list.scenes.sort();
    }
    info!("scenes: {:?}", scene_list.scenes);
}
