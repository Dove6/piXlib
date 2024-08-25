use std::sync::RwLock;

use bevy::{
    app::{App, AppExit, Plugin, Update},
    prelude::{Event, EventWriter, Events, NonSend, ResMut},
};

use pixlib_parser::runner::{
    ApplicationEvent, FileEvent, GraphicsEvent, ObjectEvent, ScriptEvent, SoundEvent,
};

use super::scripts_plugin::ScriptRunner;

#[derive(Debug, Default)]
pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PixlibScriptEvent>()
            .add_event::<PixlibFileEvent>()
            .add_event::<PixlibObjectEvent>()
            .add_event::<PixlibApplicationEvent>()
            .init_resource::<Events<PixlibSoundEvent>>()
            .add_event::<PixlibGraphicsEvent>()
            .add_systems(
                Update,
                (
                    redistribute_script_events,
                    redistribute_file_events,
                    redistribute_object_events,
                    redistribute_application_events,
                    redistribute_sound_events,
                    cleanup_processed_sound_events,
                    redistribute_graphics_events,
                ),
            );
    }
}

#[derive(Event, Debug, Clone)]
pub struct PixlibScriptEvent(pub ScriptEvent);

fn redistribute_script_events(
    runner: NonSend<ScriptRunner>,
    mut writer: EventWriter<PixlibScriptEvent>,
) {
    for evt in runner.events_out.script.borrow_mut().drain(..) {
        writer.send(PixlibScriptEvent(evt));
    }
}

#[derive(Event, Debug, Clone)]
pub struct PixlibFileEvent(pub FileEvent);

fn redistribute_file_events(
    runner: NonSend<ScriptRunner>,
    mut writer: EventWriter<PixlibFileEvent>,
) {
    for evt in runner.events_out.file.borrow_mut().drain(..) {
        writer.send(PixlibFileEvent(evt));
    }
}

#[derive(Event, Debug, Clone)]
pub struct PixlibObjectEvent(pub ObjectEvent);

fn redistribute_object_events(
    runner: NonSend<ScriptRunner>,
    mut writer: EventWriter<PixlibObjectEvent>,
) {
    for evt in runner.events_out.object.borrow_mut().drain(..) {
        writer.send(PixlibObjectEvent(evt));
    }
}

#[derive(Event, Debug, Clone)]
pub struct PixlibApplicationEvent(pub ApplicationEvent);

fn redistribute_application_events(
    runner: NonSend<ScriptRunner>,
    mut writer: EventWriter<PixlibApplicationEvent>,
    mut exit_writer: EventWriter<AppExit>,
) {
    for evt in runner.events_out.app.borrow_mut().drain(..) {
        if matches!(&evt, ApplicationEvent::ApplicationExited) {
            exit_writer.send(AppExit);
        }
        writer.send(PixlibApplicationEvent(evt));
    }
}

#[derive(Event, Debug)]
pub struct PixlibSoundEvent {
    pub event: SoundEvent,
    processed: RwLock<bool>,
}

impl PixlibSoundEvent {
    pub fn new(event: SoundEvent) -> Self {
        Self {
            event,
            processed: RwLock::new(false),
        }
    }

    pub fn has_been_processed(&self) -> bool {
        *self.processed.read().unwrap()
    }

    pub fn mark_as_processed(&self) {
        *self.processed.write().unwrap() = true;
    }
}

fn redistribute_sound_events(
    runner: NonSend<ScriptRunner>,
    mut writer: EventWriter<PixlibSoundEvent>,
) {
    for evt in runner.events_out.sound.borrow_mut().drain(..) {
        // info!("Redistributing sound event {}", evt);
        writer.send(PixlibSoundEvent::new(evt));
    }
}

fn cleanup_processed_sound_events(mut events: ResMut<Events<PixlibSoundEvent>>) {
    let old_events = events.drain().collect::<Vec<_>>();
    for evt in old_events {
        if !evt.has_been_processed() {
            events.send(evt);
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct PixlibGraphicsEvent(pub GraphicsEvent);

fn redistribute_graphics_events(
    runner: NonSend<ScriptRunner>,
    mut writer: EventWriter<PixlibGraphicsEvent>,
) {
    for evt in runner.events_out.graphics.borrow_mut().drain(..) {
        writer.send(PixlibGraphicsEvent(evt));
    }
}
