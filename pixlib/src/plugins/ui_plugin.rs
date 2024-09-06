use std::sync::{Arc, RwLock};

use bevy::color::Color;
use bevy::log::error;
use bevy::prelude::Visibility;
use bevy::state::condition::in_state;
use bevy::state::state::{NextState, OnEnter};
use bevy::{
    app::{App, Plugin, Update},
    asset::{
        io::Reader, Asset, AssetApp, AssetLoader, AssetServer, Assets, AsyncReadExt, Handle,
        LoadContext,
    },
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        query::{Changed, With},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Children, Parent},
    input::ButtonInput,
    log::info,
    prelude::{default, EventReader, IntoSystemConfigs, KeyCode},
    reflect::TypePath,
    text::{Text, TextStyle},
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        widget::Button,
        AlignItems, BackgroundColor, BorderColor, Interaction, JustifyContent, Style, UiRect, Val,
    },
    window::FileDragAndDrop,
};
use pixlib_parser::common::LoggableToOption;
use pixlib_parser::filesystems::{CompressedPatch, InsertedDisk};
use pixlib_parser::runner::FileSystem;
use thiserror::Error;

use crate::filesystems::PendingHandle;
use crate::{
    filesystems::FileSystemResource,
    resources::{ChosenScene, RootEntityToDespawn},
    AppState,
};

const COLOR_RED: Color = Color::linear_rgb(1.0, 0.0, 0.0);
const COLOR_BLACK: Color = Color::linear_rgb(0.0, 0.0, 0.0);
const COLOR_WHITE: Color = Color::linear_rgb(1.0, 1.0, 1.0);

const NORMAL_BUTTON: Color = Color::linear_rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::linear_rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::linear_rgb(0.35, 0.75, 0.35);

#[derive(Debug, Default)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Blob>()
            .init_asset_loader::<BlobAssetLoader>()
            .add_systems(
                Update,
                (
                    handle_dropped_iso,
                    navigate_chooser,
                    update_chooser_labels,
                    update_arrows_visibility,
                    insert_disk_when_loaded,
                )
                    .run_if(in_state(AppState::SceneChooser)),
            )
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
                            color: Color::linear_rgb(0.9, 0.9, 0.9),
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
                            .unwrap_or("Waiting...".to_owned()),
                        TextStyle {
                            font_size: 40.0,
                            color: Color::linear_rgb(0.9, 0.9, 0.9),
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
                            color: Color::linear_rgb(0.9, 0.9, 0.9),
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
                border_color.0 = COLOR_WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = COLOR_BLACK;
            }
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = COLOR_RED;
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
    button_query: Query<(&ButtonFunctionComponent, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    if !chosen_scene.is_changed() || chosen_scene.list.is_empty() {
        return;
    }
    for (button_function, button_children) in &button_query {
        if let ButtonFunctionComponent::Display { offset } = button_function {
            let mut text = text_query.get_mut(button_children[0]).unwrap();
            let displayed_index = chosen_scene.index + offset;
            text.sections[0].value = chosen_scene.list[displayed_index % chosen_scene.list.len()]
                .name
                .clone();
        }
    }
}

pub fn update_arrows_visibility(
    chosen_scene: Res<ChosenScene>,
    mut button_bundle_query: Query<(&ButtonFunctionComponent, &mut Visibility)>,
) {
    if !chosen_scene.is_changed() {
        return;
    }
    for (button_function, mut visibility) in button_bundle_query.iter_mut() {
        if !matches!(button_function, ButtonFunctionComponent::Display { .. }) {
            *visibility = if chosen_scene.list.is_empty() {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
}

pub fn handle_dropped_iso(
    asset_server: Res<AssetServer>,
    mut event_reader: EventReader<FileDragAndDrop>,
    mut filesystem: ResMut<FileSystemResource>,
) {
    for event in event_reader.read() {
        info!("Drag and drop event: {:?}", event);
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            #[cfg(target_family = "wasm")]
            let path = String::from(path_buf.to_str().unwrap());
            #[cfg(not(target_family = "wasm"))]
            let path = bevy::asset::AssetPath::from_path(path_buf.as_path());
            let handle: Handle<Blob> = asset_server.load(path);
            let is_main = path_buf
                .to_str()
                .unwrap_or_default()
                .to_uppercase()
                .trim()
                .ends_with(".ISO");
            filesystem.insert_handle(PendingHandle::new(handle, is_main, is_main));
        }
    }
}

pub fn insert_disk_when_loaded(
    asset_server: Res<AssetServer>,
    mut blobs: ResMut<Assets<Blob>>,
    mut filesystem: ResMut<FileSystemResource>,
) {
    let Some(pending_handle) = filesystem.get_pending_handle() else {
        return;
    };
    match asset_server.load_state(&*pending_handle) {
        bevy::asset::LoadState::Failed(e) => {
            error!(
                "Failed to load asset {}: {}",
                (*pending_handle)
                    .path()
                    .map(|p| p.to_string())
                    .unwrap_or_default(),
                e
            );
            filesystem.set_as_failed(&pending_handle).unwrap();
        }
        bevy::asset::LoadState::Loaded => {}
        _ => return,
    }
    let Some(loaded_asset) = blobs.remove(&*pending_handle) else {
        info!("Asset not loaded yet for handle {:?}", pending_handle);
        return;
    };
    let loaded_fs = if pending_handle.is_main() {
        InsertedDisk::new(loaded_asset.into_inner())
            .map(|l| -> Arc<RwLock<dyn FileSystem>> { Arc::new(RwLock::new(l)) })
            .ok_or_error()
    } else {
        CompressedPatch::new(loaded_asset.into_inner())
            .map(|l| -> Arc<RwLock<dyn FileSystem>> { Arc::new(RwLock::new(l)) })
            .ok_or_error()
    };
    if let Some(loaded_fs) = loaded_fs {
        filesystem
            .insert_loaded(&pending_handle, loaded_fs)
            .unwrap();
    } else {
        error!("Could not load filesystem");
        filesystem.set_as_failed(&pending_handle).unwrap();
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

#[derive(Asset, TypePath, Debug)]
pub struct Blob {
    bytes: Vec<u8>,
}

impl Blob {
    pub fn into_inner(self) -> Vec<u8> {
        self.bytes
    }
}

#[derive(Default)]
struct BlobAssetLoader;

/// Possible errors that can be produced by [`BlobAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for BlobAssetLoader {
    type Asset = Blob;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading Blob...");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(Blob { bytes })
    }
}
