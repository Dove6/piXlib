use std::{
    collections::HashSet,
    io::Cursor,
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{Assets, Handle},
    log::{error, info},
    prelude::{
        in_state, BuildChildren, Bundle, Commands, Component, Condition, EventReader, EventWriter,
        IntoSystemConfigs, NonSend, OnExit, Query, Res, ResMut, SpatialBundle,
    },
};
use bevy_kira_audio::{
    prelude::StaticSoundData, Audio, AudioControl, AudioInstance, AudioSource, AudioTween,
    PlaybackState,
};
use pixlib_parser::runner::{
    classes::{self, Scene},
    MultimediaEvents, ScriptEvent, SoundEvent, SoundSource,
};

use crate::AppState;

use super::{
    events_plugin::{PixlibScriptEvent, PixlibSoundEvent, PostponedPixlibSoundEvent},
    scripts_plugin::ScriptRunner,
};

const POOL_SIZE: usize = 50;
const EASING: AudioTween = AudioTween::linear(Duration::ZERO);

#[derive(Debug, Default)]
pub struct SoundsPlugin;

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_pool)
            // .add_systems(Update, update_bgm.run_if(in_state(AppState::SceneViewer)))
            .add_systems(
                Update,
                update_sounds.run_if(in_state(AppState::SceneViewer)),
            )
            // .add_systems(
            //     Update,
            //     update_animations.run_if(in_state(AppState::SceneViewer)),
            // )
            .add_systems(
                Update,
                check_for_state_transitions.run_if(in_state(AppState::SceneViewer)),
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
pub struct SoundsMarker(Option<SoundSource>);

impl Deref for SoundsMarker {
    type Target = Option<SoundSource>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SoundsMarker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component, Debug, Default)]
pub struct LoadedSoundsIdentifier(pub Option<u64>);

impl Deref for LoadedSoundsIdentifier {
    type Target = Option<u64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LoadedSoundsIdentifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component, Debug, Default)]
struct SoundsPoolMarker {
    pub state: PoolState,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum PoolState {
    #[default]
    Reset,
    Assigned,
}

#[derive(Bundle, Default)]
pub struct SoundsBundle {
    pub marker: SoundsMarker,
    pub identifier: LoadedSoundsIdentifier,
    handle: SoundsInstanceHandle,
    previous_state: SoundsState,
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

#[derive(Component, Debug, Clone, Copy, Default)]
struct SoundsState {
    pub position: Option<f64>,
}

impl SoundsState {
    pub fn has_position_after(&self, other: &Self) -> bool {
        let Some(current) = self.position else {
            return false;
        };
        let Some(other) = other.position else {
            return false;
        };
        current > other
    }
}

impl From<PlaybackState> for SoundsState {
    fn from(value: PlaybackState) -> Self {
        Self {
            position: match value {
                PlaybackState::Paused { position } => Some(position),
                PlaybackState::Pausing { position } => Some(position),
                PlaybackState::Playing { position } => Some(position),
                PlaybackState::Stopping { position } => Some(position),
                _ => None,
            },
        }
    }
}

pub fn create_pool(mut commands: Commands) {
    commands
        .spawn((SoundsPoolMarker::default(), SpatialBundle::default()))
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
        // info!("Popped event: {:?}", evt);
        if matches!(evt.0, ScriptEvent::ScriptLoaded { .. }) {
            any_script_loaded = true;
        }
    }
    any_script_loaded
}

fn reset_pool(
    mut pool_query: Query<&mut SoundsPoolMarker>,
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
        **marker = None;
        ident.0 = None;
        if let Some(handle) = handle.take() {
            if let Some(mut instance) = audio_instances.remove(handle) {
                instance.stop(EASING);
            }
        }
    }
    pool_query.single_mut().state = PoolState::Reset;
    info!("Reset {} audio objects", counter);
}

fn assign_pool(
    mut pool_query: Query<&mut SoundsPoolMarker>,
    mut query: Query<&mut SoundsMarker>,
    runner: NonSend<ScriptRunner>,
) {
    let mut bgm_assigned = false;
    let mut sound_counter = 0;
    let animation_sfx_counter = 0;
    let mut iter = query.iter_mut();
    // info!("Current scene: {:?}", runner.get_current_scene());
    if let Some(current_scene) = runner.get_current_scene() {
        let current_scene_guard = current_scene.content.borrow();
        let current_scene: Option<&Scene> = (&*current_scene_guard).into();
        let current_scene = current_scene.unwrap();
        if current_scene.has_background_music() {
            **iter.next().unwrap() = Some(SoundSource::BackgroundMusic);
            bgm_assigned = true;
        }
    }
    for script in runner.scripts.borrow().iter() {
        for object in script
            .objects
            .borrow()
            .iter()
            .filter(|o| Into::<Option<&classes::Sound>>::into(&*o.content.borrow()).is_some())
        {
            **iter.next().unwrap() = Some(SoundSource::Sound {
                script_path: script.path.clone(),
                object_name: object.name.clone(),
            });
            sound_counter += 1;
        }
    }
    pool_query.single_mut().state = PoolState::Assigned;
    info!(
        "Assigned {} background music, {} sounds and {} animation SFX",
        if bgm_assigned { "a" } else { "no" },
        sound_counter,
        animation_sfx_counter
    );
}

fn check_for_state_transitions(
    pool_query: Query<&SoundsPoolMarker>,
    mut query: Query<(&SoundsMarker, &SoundsInstanceHandle, &mut SoundsState)>,
    runner: NonSend<ScriptRunner>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if pool_query.single().state != PoolState::Assigned {
        return;
    }
    for (marker, handle, mut state) in query.iter_mut() {
        let Some(source) = &**marker else {
            continue;
        };
        let Some(handle) = &**handle else {
            continue;
        };
        let Some(instance) = audio_instances.get_mut(handle) else {
            continue;
        };
        if state.has_position_after(&instance.state().into()) {
            instance.pause(EASING);
            let mut events = runner.events_in.multimedia.borrow_mut();
            events.push_back(MultimediaEvents::SoundFinishedPlaying(source.clone()))
        }
        *state = instance.state().into();
    }
}

fn update_sounds(
    mut reader: EventReader<PixlibSoundEvent>,
    mut writer: EventWriter<PostponedPixlibSoundEvent>,
    audio: Res<Audio>,
    pool_query: Query<&SoundsPoolMarker>,
    mut query: Query<(
        &SoundsMarker,
        &mut LoadedSoundsIdentifier,
        &mut SoundsInstanceHandle,
        &mut SoundsState,
    )>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if pool_query.single().state != PoolState::Assigned {
        return;
    }
    let mut reloaded_sources = HashSet::new();
    for evt in reader.read() {
        let evt_source = match &evt.0 {
            SoundEvent::SoundLoaded { source, .. } => source,
            SoundEvent::SoundStarted(source) => source,
            SoundEvent::SoundPaused(source) => source,
            SoundEvent::SoundResumed(source) => source,
            SoundEvent::SoundStopped(source) => source,
        };
        if reloaded_sources.contains(evt_source) {
            writer.send(PostponedPixlibSoundEvent(evt.0.clone()));
            continue;
        }
        // info!("Read sound event: {}", evt.0);
        for (marker, mut ident, mut handle, mut state) in query.iter_mut() {
            let Some(snd_source) = &**marker else {
                continue;
            };
            if evt_source != snd_source {
                continue;
            }
            // info!("Matched the sounds pool element");
            match &evt.0 {
                SoundEvent::SoundLoaded { sound_data, .. } => {
                    if !ident.is_some_and(|h| h == sound_data.hash) {
                        let source = audio_sources.add(AudioSource {
                            sound: StaticSoundData::from_cursor(
                                Cursor::new(sound_data.data.clone()),
                                Default::default(),
                            )
                            .unwrap(),
                        });
                        let new_handle: Handle<AudioInstance> =
                            audio.play(source).looped().paused().handle();
                        if let Some(handle) = handle.replace(new_handle) {
                            if let Some(mut instance) = audio_instances.remove(handle) {
                                instance.stop(EASING);
                            }
                        }
                        ident.0 = Some(sound_data.hash);
                        state.position = Some(0.0);
                        reloaded_sources.insert(evt_source.clone());
                        // info!("Updated data for sound {:?}", snd_source);
                    }
                }
                _ => {
                    let Some(instance) =
                        (*handle).as_ref().and_then(|h| audio_instances.get_mut(h))
                    else {
                        error!("Cannot retrieve audio instance for sound {:?}", snd_source);
                        break;
                    };
                    match &evt.0 {
                        SoundEvent::SoundStarted(_) => {
                            instance.resume(EASING);
                            // info!("Started sound {:?}", snd_source);
                        }
                        SoundEvent::SoundPaused(_) => {
                            instance.pause(EASING);
                            // info!("Paused sound {:?}", snd_source);
                        }
                        SoundEvent::SoundResumed(_) => {
                            instance.resume(EASING);
                            // info!("Resumed sound {:?}", snd_source);
                        }
                        SoundEvent::SoundStopped(_) => {
                            instance.pause(EASING);
                            instance.seek_to(0.0);
                            state.position = Some(0.0);
                            // info!("Stopped sound {:?}", snd_source);
                        }
                        _ => unreachable!(),
                    };
                }
            };
        }
    }
}
