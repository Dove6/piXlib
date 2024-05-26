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
    resources::{
        ChosenScene, GamePaths, InsertedDisk, RootEntityToDespawn, SceneDefinition, ScriptRunner,
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

pub fn setup_chooser(
    game_paths: Res<GamePaths>,
    inserted_disk: Res<InsertedDisk>,
    chosen_scene: Res<ChosenScene>,
    mut commands: Commands,
    mut script_runner: ResMut<ScriptRunner>,
) {
    let mut scene_list = SceneListComponent::default();
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
                            .unwrap_or("(scene name)".to_owned()),
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
) {
    scene_list.scenes.clear();
    if let Some(iso) = inserted_disk.get() {
        let game_definition_path = read_game_definition(iso, game_paths, script_runner);
        let game_definition = script_runner.0.get_script(&game_definition_path).unwrap();
        println!("game_definition: {:?}", game_definition);
        for (name, path, background) in
            game_definition
                .objects
                .iter()
                .filter_map(|o| match &o.content {
                    CnvType::Scene(scene) if scene.read().unwrap().path.is_some() => Some((
                        o.name.clone(),
                        scene.read().unwrap().path.clone(),
                        scene.read().unwrap().background.clone(),
                    )),
                    _ => None,
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
    println!("scenes: {:?}", scene_list.scenes);
}
