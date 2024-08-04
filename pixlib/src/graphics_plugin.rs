use std::{path::Path, sync::Arc};

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{Assets, Handle},
    log::{error, info},
    prelude::{
        in_state, BuildChildren, Bundle, Commands, Component, DespawnRecursiveExt, Entity, Image,
        IntoSystemConfigs, NonSend, Query, ResMut, SpatialBundle, Transform, Visibility,
    },
    sprite::{Sprite, SpriteBundle},
};
use pixlib_formats::file_formats::img::parse_img;
use pixlib_parser::{
    classes::{CnvObject, PropertyValue},
    runner::ScriptEvent,
};

use crate::{
    anchors::add_tuples,
    resources::ScriptRunner,
    states::AppState,
    util::{animation_data_to_handle, image_data_to_handle, img_file_to_handle},
};

const POOL_SIZE: usize = 50;

#[derive(Debug, Default)]
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        // app.init_resource::<MyOtherResource>();
        // app.add_event::<MyEvent>();
        app.add_systems(Startup, create_pool)
            .add_systems(
                Update,
                update_background.run_if(in_state(AppState::SceneViewer)),
            )
            .add_systems(
                Update,
                update_images.run_if(in_state(AppState::SceneViewer)),
            )
            .add_systems(
                Update,
                update_animations.run_if(in_state(AppState::SceneViewer)),
            )
            .add_systems(Update, assign_pool.run_if(in_state(AppState::SceneViewer)));
    }
}

#[derive(Component, Debug, Default)]
pub enum GraphicsMarker {
    #[default]
    Unassigned,
    BackgroundImage,
    Image {
        script_path: Arc<Path>,
        index: usize,
    },
    Animation {
        script_path: Arc<Path>,
        index: usize,
    },
}

#[derive(Component, Debug, Default)]
pub struct GraphicsPoolMarker;

#[derive(Bundle, Default)]
pub struct GraphicsBundle {
    pub marker: GraphicsMarker,
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

pub fn reset_pool(
    mut query: Query<(
        &mut GraphicsMarker,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
) {
    let mut counter = 0;
    for (mut marker, mut sprite, mut transform, mut handle, mut visibility) in query.iter_mut() {
        counter += 1;
        *marker = GraphicsMarker::Unassigned;
        sprite.flip_x = false;
        sprite.flip_y = false;
        *transform = Transform::from_xyz(0f32, 0f32, 0f32);
        *handle = Handle::default();
        *visibility = Visibility::Hidden;
    }
    info!("Reset {} graphics objects", counter);
}

pub fn assign_pool(mut query: Query<&mut GraphicsMarker>, runner: NonSend<ScriptRunner>) {
    let mut out_events = runner.events_out.script.borrow_mut();
    let mut any_script_loaded = false;
    while let Some(ScriptEvent::ScriptLoaded { .. }) = out_events.front() {
        info!("Popped event: {:?}", out_events.pop_front());
        any_script_loaded = true;
    }
    if !any_script_loaded {
        return;
    }
    let mut all_objects = Vec::new();
    runner.find_objects(|_| true, &mut all_objects);
    let all_objects: Vec<String> = all_objects.iter().map(|o| o.name.clone()).collect();
    info!("All loaded objects: {:?}", all_objects);
    let mut background_assigned = false;
    let mut image_counter = 0;
    let mut animation_counter = 0;
    let mut iter = query.iter_mut();
    if let Some(current_scene) = runner.get_current_scene() {
        if current_scene.get_property("BACKGROUND").is_some() {
            *iter.next().unwrap() = GraphicsMarker::BackgroundImage;
            background_assigned = true;
        }
    }
    let mut scene_graphics: Vec<Arc<CnvObject>> = Vec::new();
    runner.find_objects(
        |o| matches!(o.content.borrow().as_ref().unwrap().get_type_id(), "IMAGE"),
        &mut scene_graphics,
    );
    for graphics_object_index in scene_graphics.iter() {
        let mut marker = iter.next().unwrap();
        *marker = GraphicsMarker::Image {
            script_path: graphics_object_index.parent.path.clone(),
            index: graphics_object_index.index,
        };
        image_counter += 1;
    }
    runner.find_objects(
        |o| matches!(o.content.borrow().as_ref().unwrap().get_type_id(), "ANIMO"),
        &mut scene_graphics,
    );
    for graphics_object_index in scene_graphics.iter() {
        let mut marker = iter.next().unwrap();
        *marker = GraphicsMarker::Animation {
            script_path: graphics_object_index.parent.path.clone(),
            index: graphics_object_index.index,
        };
        animation_counter += 1;
    }
    info!(
        "Assigned {} background, {} images and {} animations",
        if background_assigned { "a" } else { "no" },
        image_counter,
        animation_counter
    );
}

pub fn update_background(
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<(
        &mut GraphicsMarker,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    runner: NonSend<ScriptRunner>,
) {
    for (marker, mut sprite, mut transform, mut handle, mut visibility) in query.iter_mut() {
        if !matches!(*marker, GraphicsMarker::BackgroundImage) {
            continue;
        }
        let PropertyValue::String(scene_path) = runner
            .get_current_scene()
            .unwrap()
            .get_property("PATH")
            .unwrap()
        else {
            continue;
        };
        let PropertyValue::String(background_path) = runner
            .get_current_scene()
            .unwrap()
            .get_property("BACKGROUND")
            .unwrap()
        else {
            continue;
        };
        let loaded_file = (*runner.filesystem)
            .borrow()
            .read_file(
                &Path::new(&scene_path)
                    .with_file_name(background_path)
                    .as_os_str()
                    .to_str()
                    .unwrap(),
            )
            .unwrap();
        let image = parse_img(&loaded_file);
        *handle = img_file_to_handle(&mut textures, image);
        *visibility = Visibility::Visible;
        sprite.flip_x = false;
        sprite.flip_y = false;
        *transform = Transform::from_xyz(0f32, 0f32, 0f32);
        info!("Updated background for scene {}", scene_path);
    }
}

pub fn update_images(
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<(
        &mut GraphicsMarker,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    runner: NonSend<ScriptRunner>,
) {
    for (marker, mut sprite, mut transform, mut handle, mut visibility) in query.iter_mut() {
        let GraphicsMarker::Image { script_path, index } = &*marker else {
            continue;
        };
        let Some(script) = runner.get_script(&script_path) else {
            continue;
        };
        let Some(object) = script.objects.borrow().get_object_at(*index) else {
            continue;
        };
        let image_guard = object.content.borrow();
        let Some(image) = image_guard
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<pixlib_parser::classes::Image>()
        else {
            continue;
        };

        sprite.flip_x = false;
        sprite.flip_y = false;
        let Some((image_definition, image_data)) = image.get_image_to_show().unwrap() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = if image.is_visible() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *transform = Transform::from_xyz(
            image_definition.offset_px.0 as f32,
            image_definition.offset_px.1 as f32,
            0f32,
        );
        *handle = image_data_to_handle(&mut textures, image_definition, image_data);
        info!("Updated image {}", &object.name);
    }
}

pub fn update_animations(
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<(
        &mut GraphicsMarker,
        &mut Sprite,
        &mut Transform,
        &mut Handle<Image>,
        &mut Visibility,
    )>,
    runner: NonSend<ScriptRunner>,
) {
    for (marker, _sprite, mut transform, mut handle, mut visibility) in query.iter_mut() {
        let GraphicsMarker::Animation { script_path, index } = &*marker else {
            continue;
        };
        let Some(script) = runner.get_script(&script_path) else {
            continue;
        };
        let Some(object) = script.objects.borrow().get_object_at(*index) else {
            continue;
        };
        let animation_guard = object.content.borrow();
        let Some(animation) = animation_guard
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<pixlib_parser::classes::Animation>()
        else {
            continue;
        };

        let Ok(frame_to_show) = animation
            .get_frame_to_show()
            .inspect_err(|e| error!("Error getting frame to show: {:?}", e))
        else {
            continue;
        };
        let Some((frame, sprite, data)) = frame_to_show else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = if animation.is_visible() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let total_offset = add_tuples(sprite.offset_px, frame.offset_px);
        //sprite.flip_x = animation
        *transform = Transform::from_xyz(total_offset.0 as f32, total_offset.1 as f32, 0f32);
        *handle = animation_data_to_handle(&mut textures, &sprite, &data);
        info!("Updated animation {}", &object.name);
    }
}

pub fn destroy_pool(mut commands: Commands, query: Query<(Entity, &GraphicsPoolMarker)>) {
    if let Some((entity, _)) = query.iter().next() {
        commands.entity(entity).despawn_recursive();
        info!("Destroyed the pool");
    };
}
