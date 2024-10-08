use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{Assets, Handle},
    log::{error, info},
    math::Vec3,
    prelude::{
        in_state, BuildChildren, Bundle, Commands, Component, Condition, EventReader, Image,
        IntoSystemConfigs, NonSend, OnExit, Query, ResMut, SpatialBundle, Transform, Visibility,
    },
    sprite::{Anchor, Sprite, SpriteBundle},
};

use pixlib_parser::runner::{classes::GeneralGraphics, CnvContent, ScenePath, ScriptEvent};

use crate::{
    util::{animation_data_to_handle, image_data_to_handle},
    AppState,
};

use super::{events_plugin::PixlibScriptEvent, scripts_plugin::ScriptRunner};

const POOL_SIZE: usize = 1111;

#[derive(Debug, Default)]
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_pool)
            .add_systems(
                Update,
                (update_background, update_images, update_animations)
                    .run_if(in_state(AppState::SceneViewer)),
            )
            .add_systems(
                Update,
                (reset_pool, assign_pool)
                    .chain()
                    .run_if(in_state(AppState::SceneViewer).and_then(run_if_any_script_loaded)),
            )
            .add_systems(OnExit(AppState::SceneViewer), reset_pool);
    }
}

#[derive(Component, Debug, Default, Clone)]
pub enum GraphicsMarker {
    #[default]
    Unassigned,
    BackgroundImage,
    Image {
        script_index: usize,
        script_path: ScenePath,
        object_index: usize,
        object_name: String,
    },
    Animation {
        script_index: usize,
        script_path: ScenePath,
        object_index: usize,
        object_name: String,
    },
}

#[derive(Component, Debug, Default)]
pub struct LoadedGraphicsIdentifier(pub Option<u64>);

#[derive(Component, Debug, Default)]
pub struct GraphicsPoolMarker;

#[derive(Bundle, Default)]
pub struct GraphicsBundle {
    pub marker: GraphicsMarker,
    pub identifier: LoadedGraphicsIdentifier,
    pub sprite: SpriteBundle,
}

pub fn create_pool(mut commands: Commands) {
    commands
        .spawn((GraphicsPoolMarker, SpatialBundle::default()))
        .with_children(|parent| {
            for _ in 0..POOL_SIZE {
                parent.spawn(GraphicsBundle::default());
            }
        });
    info!("Created a pool of {} graphics objects", POOL_SIZE);
}

fn run_if_any_script_loaded(mut reader: EventReader<PixlibScriptEvent>) -> bool {
    let mut any_script_loaded = false;
    for evt in reader.read() {
        // info!("Popped event: {:?}", evt);
        if matches!(evt.0, ScriptEvent::ScriptLoaded { .. }) {
            any_script_loaded = true;
        }
    }
    any_script_loaded
}

pub fn reset_pool(
    mut query: Query<(
        &mut GraphicsMarker,
        &mut LoadedGraphicsIdentifier,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
) {
    let mut counter = 0;
    for (mut marker, mut ident, mut sprite, mut transform, mut handle, mut visibility) in
        query.iter_mut()
    {
        counter += 1;
        *marker = GraphicsMarker::Unassigned;
        ident.0 = None;
        sprite.flip_x = false;
        sprite.flip_y = false;
        sprite.anchor = Anchor::TopLeft;
        *transform = Transform::from_xyz(0f32, 0f32, 0f32);
        *handle = Handle::default();
        *visibility = Visibility::Hidden;
    }
    info!("Reset {} graphics objects", counter);
}

pub fn assign_pool(mut query: Query<&mut GraphicsMarker>, runner: NonSend<ScriptRunner>) {
    // let mut all_objects = Vec::new();
    // runner.find_objects(|_| true, &mut all_objects);
    // let all_objects: Vec<String> = all_objects.iter().map(|o| o.name.clone()).collect();
    // info!("All loaded objects: {:?}", all_objects);
    let mut image_counter = 0;
    let mut animation_counter = 0;
    let mut iter = query.iter_mut();
    // info!("Current scene: {:?}", runner.get_current_scene());
    *iter.next().unwrap() = GraphicsMarker::BackgroundImage;
    for (script_index, script) in runner.scripts.borrow().iter().enumerate() {
        for (object_index, object) in script.objects.borrow().iter().enumerate() {
            if !matches!(&object.content, CnvContent::Image(_)) {
                continue;
            }
            let mut marker = iter.next().unwrap();
            *marker = GraphicsMarker::Image {
                script_index,
                script_path: script.path.clone(),
                object_index,
                object_name: object.name.clone(),
            };
            image_counter += 1;
        }
    }
    for (script_index, script) in runner.scripts.borrow().iter().enumerate() {
        for (object_index, object) in script.objects.borrow().iter().enumerate() {
            if !matches!(&object.content, CnvContent::Animation(_)) {
                continue;
            }
            let mut marker = iter.next().unwrap();
            *marker = GraphicsMarker::Animation {
                script_index,
                script_path: script.path.clone(),
                object_index,
                object_name: object.name.clone(),
            };
            animation_counter += 1;
        }
    }
    info!(
        "Assigned {} images and {} animations",
        image_counter, animation_counter
    );
}

pub fn update_background(
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<(
        &GraphicsMarker,
        &mut LoadedGraphicsIdentifier,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    runner: NonSend<ScriptRunner>,
) {
    for (marker, mut ident, mut sprite, mut transform, mut handle, mut visibility) in
        query.iter_mut()
    {
        if !matches!(*marker, GraphicsMarker::BackgroundImage) {
            continue;
        }
        // info!("Current scene: {:?}", runner.get_current_scene());
        let Some(canvas_observer_object) =
            runner.find_object(|o| matches!(&o.content, CnvContent::CanvasObserver(_)))
        else {
            continue;
        };
        let CnvContent::CanvasObserver(ref canvas_observer) = &canvas_observer_object.content
        else {
            unreachable!();
        };
        let result = canvas_observer.get_background_to_show();
        let Ok(background_data) = result else {
            error!(
                "Error getting background image for scene {}: {:?}",
                canvas_observer_object.name,
                result.unwrap_err()
            );
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some((_, image_definition, image_data)) = background_data else {
            *visibility = Visibility::Hidden;
            continue;
        };
        sprite.flip_x = false;
        sprite.flip_y = false;
        sprite.anchor = Anchor::TopLeft;
        *visibility = Visibility::Visible;
        *transform = Transform::from_xyz(
            image_definition.offset_px.0 as f32,
            image_definition.offset_px.1 as f32,
            -995f32,
        )
        .with_scale(Vec3::new(1f32, -1f32, 1f32));
        if !ident.0.is_some_and(|h| h == image_data.hash) {
            *handle = image_data_to_handle(&mut textures, &image_definition, &image_data);
            ident.0 = Some(image_data.hash);
            // info!(
            //     "Updated background for scene {:?} / {:?}",
            //     scene.get_script_path(), scene_object.name
            // );
        }
    }
}

pub fn update_images(
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<(
        &GraphicsMarker,
        &mut LoadedGraphicsIdentifier,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    runner: NonSend<ScriptRunner>,
) {
    for (marker, mut ident, mut sprite, mut transform, mut handle, mut visibility) in
        query.iter_mut()
    {
        let GraphicsMarker::Image {
            script_index,
            script_path,
            object_index,
            object_name: _,
        } = marker
        else {
            continue;
        };
        let Some(script) = runner.get_script(script_path) else {
            continue;
        };
        let Some(object) = script.objects.borrow().get_object_at(*object_index) else {
            continue;
        };
        let CnvContent::Image(ref image) = &object.content else {
            continue;
        };

        let Some((image_definition, image_data)) = image.get_image_to_show().unwrap() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = if image.is_visible().unwrap() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        sprite.flip_x = false;
        sprite.flip_y = false;
        sprite.anchor = Anchor::TopLeft;
        let position = image.get_position().unwrap();
        *transform = Transform::from_xyz(
            position.0 as f32,
            position.1 as f32,
            image.get_priority().unwrap() as f32
                + (*script_index as f32) / 100f32
                + (*object_index as f32) / 100000f32,
        )
        .with_scale(Vec3::new(1f32, -1f32, 1f32));
        if !ident.0.is_some_and(|h| h == image_data.hash) {
            *handle = image_data_to_handle(&mut textures, &image_definition, &image_data);
            ident.0 = Some(image_data.hash);
            // info!(
            //     "Updated image {} with priority {}",
            //     &object.name,
            //     image.get_priority().unwrap()
            // );
        }
    }
}

pub fn update_animations(
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<(
        &GraphicsMarker,
        &mut LoadedGraphicsIdentifier,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    runner: NonSend<ScriptRunner>,
) {
    for (marker, mut ident, mut sprite, mut transform, mut handle, mut visibility) in
        query.iter_mut()
    {
        let GraphicsMarker::Animation {
            script_index,
            script_path,
            object_index,
            object_name: _,
        } = marker
        else {
            continue;
        };
        let Some(script) = runner.get_script(script_path) else {
            continue;
        };
        let Some(object) = script.objects.borrow().get_object_at(*object_index) else {
            continue;
        };
        let CnvContent::Animation(ref animation) = &object.content else {
            continue;
        };

        let Ok(frame_to_show) = animation
            .get_frame_to_show()
            .inspect_err(|e| error!("Error getting frame to show: {:?}", e))
        else {
            continue;
        };
        let Some((rect, sprite_data)) = frame_to_show else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = if animation.is_visible().unwrap() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        sprite.flip_x = false;
        sprite.flip_y = false;
        sprite.anchor = Anchor::TopLeft;
        *transform = Transform::from_xyz(
            rect.top_left_x as f32,
            rect.top_left_y as f32,
            animation.get_priority().unwrap() as f32
                + (*script_index as f32) / 100f32
                + (*object_index as f32) / 100000f32,
        )
        .with_scale(Vec3::new(1f32, -1f32, 1f32));
        if !ident.0.is_some_and(|h| h == sprite_data.hash) {
            *handle = animation_data_to_handle(&mut textures, rect, &sprite_data);
            ident.0 = Some(sprite_data.hash);
            // info!(
            //     "Updated animation {} with priority {} to position ({}, {})+({}, {})+({}, {})",
            //     &object.name,
            //     animation.get_priority().unwrap(),
            //     base_position.0,
            //     base_position.1,
            //     sprite_definition.offset_px.0,
            //     sprite_definition.offset_px.1,
            //     frame_definition.offset_px.0,
            //     frame_definition.offset_px.1,
            // );
        }
    }
}
