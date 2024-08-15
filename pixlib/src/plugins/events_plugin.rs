use bevy::{
    app::{App, Plugin, Update},
    log::{info, warn},
    prelude::{Event, EventReader, EventWriter, NonSend},
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
            .add_event::<PixlibSoundEvent>()
            .add_event::<PostponedPixlibSoundEvent>()
            .add_event::<PixlibGraphicsEvent>()
            .add_systems(
                Update,
                (
                    redistribute_script_events,
                    redistribute_file_events,
                    redistribute_object_events,
                    redistribute_application_events,
                    redistribute_sound_events,
                    re_redistribute_sound_events,
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
) {
    for evt in runner.events_out.app.borrow_mut().drain(..) {
        writer.send(PixlibApplicationEvent(evt));
    }
}

#[derive(Event, Debug, Clone)]
pub struct PixlibSoundEvent(pub SoundEvent);

#[derive(Event, Debug, Clone)]
pub struct PostponedPixlibSoundEvent(pub SoundEvent);

fn redistribute_sound_events(
    runner: NonSend<ScriptRunner>,
    mut writer: EventWriter<PixlibSoundEvent>,
) {
    for evt in runner.events_out.sound.borrow_mut().drain(..) {
        info!("Redistributing sound event {}", evt);
        writer.send(PixlibSoundEvent(evt));
    }
}

fn re_redistribute_sound_events(
    mut re_reader: EventReader<PostponedPixlibSoundEvent>,
    mut re_writer: EventWriter<PixlibSoundEvent>,
) {
    for (evt, evt_id) in re_reader.read_with_id() {
        if evt_id.id > 100 {
            warn!("Postponed event ID: {}", evt_id.id);
        };
        warn!("Re-redistributing sound event {}", evt.0);
        re_writer.send(PixlibSoundEvent(evt.0.clone()));
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
