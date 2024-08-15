use std::{
    io::Cursor,
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{Assets, Handle},
    log::info,
    prelude::{
        in_state, BuildChildren, Bundle, Commands, Component, Condition, DespawnRecursiveExt,
        Entity, EventReader, IntoSystemConfigs, NonSend, OnExit, Query, Res, ResMut, SpatialBundle,
    },
};
use bevy_kira_audio::{
    prelude::StaticSoundData, Audio, AudioControl, AudioInstance, AudioSource, AudioTween,
};
use pixlib_parser::runner::{classes::Scene, CnvContent, ScenePath, ScriptEvent};

use crate::AppState;

use super::{events_plugin::PixlibScriptEvent, scripts_plugin::ScriptRunner};

const POOL_SIZE: usize = 50;
const EASING: AudioTween = AudioTween::linear(Duration::ZERO);

#[derive(Debug, Default)]
pub struct SoundsPlugin;

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_pool)
            .add_systems(
                Update,
                update_background.run_if(in_state(AppState::SceneViewer)),
            )
            // .add_systems(
            //     Update,
            //     update_images.run_if(in_state(AppState::SceneViewer)),
            // )
            // .add_systems(
            //     Update,
            //     update_animations.run_if(in_state(AppState::SceneViewer)),
            // )
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
pub enum SoundsMarker {
    #[default]
    Unassigned,
    BackgroundMusic,
    Sound {
        script_index: usize,
        script_path: ScenePath,
        object_index: usize,
        object_name: String,
    },
    AnimationRandomSfx {
        script_index: usize,
        script_path: ScenePath,
        object_index: usize,
        object_name: String,
    },
}

#[derive(Component, Debug, Default)]
pub struct LoadedSoundsIdentifier(pub Option<u64>);

#[derive(Component, Debug, Default)]
pub struct SoundsPoolMarker;

#[derive(Bundle, Default)]
pub struct SoundsBundle {
    pub marker: SoundsMarker,
    pub identifier: LoadedSoundsIdentifier,
    handle: SoundsInstanceHandle,
}

#[derive(Component, Debug, Clone, Default)]
struct SoundsInstanceHandle(Option<Handle<AudioInstance>>);

impl Deref for SoundsInstanceHandle {
    type Target = Option<Handle<AudioInstance>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SoundsInstanceHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn create_pool(mut commands: Commands) {
    commands
        .spawn((SoundsPoolMarker, SpatialBundle::default()))
        .with_children(|parent| {
            for _ in 0..POOL_SIZE {
                parent.spawn(SoundsBundle::default());
            }
        });
    info!("Created a pool of {} audio objects", POOL_SIZE);
}

fn run_if_any_script_loaded(mut reader: EventReader<PixlibScriptEvent>) -> bool {
    let mut any_script_loaded = false;
    for evt in reader.read() {
        info!("Popped event: {:?}", evt);
        if matches!(evt.0, ScriptEvent::ScriptLoaded { .. }) {
            any_script_loaded = true;
        }
    }
    any_script_loaded
}

fn reset_pool(
    mut query: Query<(
        &mut SoundsMarker,
        &mut LoadedSoundsIdentifier,
        &mut SoundsInstanceHandle,
    )>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let mut counter = 0;
    for (mut marker, mut ident, mut handle) in query.iter_mut() {
        counter += 1;
        *marker = SoundsMarker::Unassigned;
        ident.0 = None;
        if let Some(handle) = handle.take() {
            if let Some(mut instance) = audio_instances.remove(handle) {
                instance.stop(EASING);
            }
        }
    }
    info!("Reset {} audio objects", counter);
}

fn assign_pool(mut query: Query<&mut SoundsMarker>, runner: NonSend<ScriptRunner>) {
    let mut bgm_assigned = false;
    let mut sound_counter = 0;
    let animation_sfx_counter = 0;
    let mut iter = query.iter_mut();
    info!("Current scene: {:?}", runner.get_current_scene());
    if let Some(current_scene) = runner.get_current_scene() {
        let current_scene_guard = current_scene.content.borrow();
        let current_scene: Option<&Scene> = (&*current_scene_guard).into();
        let current_scene = current_scene.unwrap();
        if current_scene.has_background_music() {
            *iter.next().unwrap() = SoundsMarker::BackgroundMusic;
            bgm_assigned = true;
        }
    }
    for (script_index, script) in runner.scripts.borrow().iter().enumerate() {
        for (object_index, object) in script.objects.borrow().iter().enumerate() {
            if !matches!(&*object.content.borrow(), CnvContent::Sound(_)) {
                continue;
            }
            let mut marker = iter.next().unwrap();
            *marker = SoundsMarker::Sound {
                script_index,
                script_path: script.path.clone(),
                object_index,
                object_name: object.name.clone(),
            };
            sound_counter += 1;
        }
    }
    info!(
        "Assigned {} background, {} sounds and {} animation SFX",
        if bgm_assigned { "a" } else { "no" },
        sound_counter,
        animation_sfx_counter
    );
}

fn update_background(
    audio: Res<Audio>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut query: Query<(
        &SoundsMarker,
        &mut LoadedSoundsIdentifier,
        &mut SoundsInstanceHandle,
    )>,
    runner: NonSend<ScriptRunner>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for (marker, mut ident, mut handle) in query.iter_mut() {
        if !matches!(*marker, SoundsMarker::BackgroundMusic) {
            continue;
        }
        // info!("Current scene: {:?}", runner.get_current_scene());
        let Some(scene_object) = runner.get_current_scene() else {
            continue;
        };
        let scene_guard = scene_object.content.borrow_mut();
        let scene: Option<&Scene> = (&*scene_guard).into();
        let scene = scene.unwrap();
        let scene_script_path = scene.get_script_path();
        let Ok(sound_data) = scene.get_music_to_play() else {
            eprintln!(
                "Error getting background music for scene {}",
                scene_object.name
            );
            if let Some(handle) = handle.take() {
                if let Some(mut instance) = audio_instances.remove(handle) {
                    instance.stop(EASING);
                }
            }
            continue;
        };
        if !ident.0.is_some_and(|h| h == sound_data.hash) {
            let source = audio_sources.add(AudioSource {
                sound: StaticSoundData::from_cursor(
                    Cursor::new(sound_data.data),
                    Default::default(),
                )
                .unwrap(),
            });
            let new_handle: Handle<AudioInstance> = audio.play(source).looped().handle();
            if let Some(handle) = handle.replace(new_handle) {
                if let Some(mut instance) = audio_instances.remove(handle) {
                    instance.stop(EASING);
                }
            }
            ident.0 = Some(sound_data.hash);
            info!(
                "Updated music for scene {:?} / {:?}",
                scene_script_path, scene_object.name
            );
        }
    }
}

// pub fn update_sounds(
//     reader: EventWriter<PixlibSoundEvent>,
//     audio: Res<Audio>,
//     mut query: Query<(
//         &SoundsMarker,
//         &mut LoadedSoundsIdentifier,
//         &mut SoundsInstanceHandle,
//     )>,
//     runner: NonSend<ScriptRunner>,
//     mut audio_sources: ResMut<Assets<AudioSource>>,
//     mut audio_instances: ResMut<Assets<AudioInstance>>,
// ) {
//     for (marker, mut ident) in query.iter_mut() {
//         let SoundsMarker::Image {
//             script_index,
//             script_path,
//             object_index,
//             object_name: _,
//         } = marker
//         else {
//             continue;
//         };
//         let Some(script) = runner.get_script(script_path) else {
//             continue;
//         };
//         let Some(object) = script.objects.borrow().get_object_at(*object_index) else {
//             continue;
//         };
//         let image_guard = object.content.borrow();
//         let image: Option<&classes::Image> = (&*image_guard).into();
//         let Some(image) = image else {
//             continue;
//         };

//         let Some((image_definition, image_data)) = image.get_image_to_show().unwrap() else {
//             *visibility = Visibility::Hidden;
//             continue;
//         };
//         *visibility = if image.is_visible().unwrap() {
//             Visibility::Visible
//         } else {
//             Visibility::Hidden
//         };
//         sprite.flip_x = false;
//         sprite.flip_y = false;
//         sprite.anchor = Anchor::TopLeft;
//         let position = image.get_position().unwrap();
//         *transform = Transform::from_xyz(
//             position.0 as f32,
//             position.1 as f32,
//             image.get_priority().unwrap() as f32
//                 + (*script_index as f32) / 100f32
//                 + (*object_index as f32) / 100000f32,
//         )
//         .with_scale(Vec3::new(1f32, -1f32, 1f32));
//         if !ident.0.is_some_and(|h| h == image_data.hash) {
//             *handle = image_data_to_handle(&mut textures, &image_definition, &image_data);
//             ident.0 = Some(image_data.hash);
//             info!(
//                 "Updated image {} with priority {}",
//                 &object.name,
//                 image.get_priority().unwrap()
//             );
//         }
//     }
// }

// pub fn update_animations(
//     mut textures: ResMut<Assets<Image>>,
//     mut query: Query<(&SoundsMarker, &mut LoadedSoundsIdentifier)>,
//     runner: NonSend<ScriptRunner>,
// ) {
//     for (marker, mut ident) in query.iter_mut() {
//         // let SoundsMarker::Animation {
//         //     script_index,
//         //     script_path,
//         //     object_index,
//         //     object_name: _,
//         // } = marker
//         // else {
//         //     continue;
//         // };
//         // let Some(script) = runner.get_script(script_path) else {
//         //     continue;
//         // };
//         // let Some(object) = script.objects.borrow().get_object_at(*object_index) else {
//         //     continue;
//         // };
//         // let animation_guard = object.content.borrow();
//         // let animation: Option<&classes::Animation> = (&*animation_guard).into();
//         // let Some(animation) = animation else {
//         //     continue;
//         // };

//         // let Ok(frame_to_show) = animation
//         //     .get_frame_to_show()
//         //     .inspect_err(|e| error!("Error getting frame to show: {:?}", e))
//         // else {
//         //     continue;
//         // };
//         // let Some((frame_definition, sprite_definition, sprite_data)) = frame_to_show else {
//         //     *visibility = Visibility::Hidden;
//         //     continue;
//         // };
//         // *visibility = if animation.is_visible().unwrap() {
//         //     Visibility::Visible
//         // } else {
//         //     Visibility::Hidden
//         // };
//         // let total_offset = add_tuples(sprite_definition.offset_px, frame_definition.offset_px);
//         // sprite.flip_x = false;
//         // sprite.flip_y = false;
//         // sprite.anchor = Anchor::TopLeft;
//         // let base_position = animation.get_base_position().unwrap();
//         // *transform = Transform::from_xyz(
//         //     base_position.0 as f32 + total_offset.0 as f32,
//         //     base_position.1 as f32 + total_offset.1 as f32,
//         //     animation.get_priority().unwrap() as f32
//         //         + (*script_index as f32) / 100f32
//         //         + (*object_index as f32) / 100000f32,
//         // )
//         // .with_scale(Vec3::new(1f32, -1f32, 1f32));
//         // if !ident.0.is_some_and(|h| h == sprite_data.hash) {
//         //     *handle = animation_data_to_handle(&mut textures, &sprite_definition, &sprite_data);
//         //     ident.0 = Some(sprite_data.hash);
//         //     info!(
//         //         "Updated animation {} with priority {} to position ({}, {})+({}, {})+({}, {})",
//         //         &object.name,
//         //         animation.get_priority().unwrap(),
//         //         base_position.0,
//         //         base_position.1,
//         //         sprite_definition.offset_px.0,
//         //         sprite_definition.offset_px.1,
//         //         frame_definition.offset_px.0,
//         //         frame_definition.offset_px.1,
//         //     );
//         // }
//     }
// }

pub fn destroy_pool(mut commands: Commands, query: Query<(Entity, &SoundsPoolMarker)>) {
    if let Some((entity, _)) = query.iter().next() {
        commands.entity(entity).despawn_recursive();
        info!("Destroyed the pool");
    };
}
