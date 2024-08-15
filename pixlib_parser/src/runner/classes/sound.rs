use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_bool, parse_event_handler};

use crate::{
    common::DroppableRefMut,
    parser::ast::ParsedScript,
    runner::{InternalEvent, SoundEvent},
};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct SoundProperties {
    // SOUND
    pub filename: Option<String>,         // FILENAME
    pub flush_after_played: Option<bool>, // FLUSHAFTERPLAYED
    pub preload: Option<bool>,            // PRELOAD

    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_finished: Option<Arc<ParsedScript>>, // ONFINISHED signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_resumed: Option<Arc<ParsedScript>>, // ONRESUMED signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_started: Option<Arc<ParsedScript>>, // ONSTARTED signal
}

#[derive(Debug, Clone, Default)]
struct SoundState {
    pub initialized: bool,

    // initialized from properties
    pub file_data: SoundFileData,

    // deduced from methods
    pub is_playing: bool,
    pub is_paused: bool,
    pub music_frequency: usize,
    pub music_volume: f32,
    pub music_pan: f32,
}

#[derive(Debug, Clone)]
pub struct SoundEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>,     // ONDONE signal
    pub on_finished: Option<Arc<ParsedScript>>, // ONFINISHED signal
    pub on_init: Option<Arc<ParsedScript>>,     // ONINIT signal
    pub on_resumed: Option<Arc<ParsedScript>>,  // ONRESUMED signal
    pub on_signal: Option<Arc<ParsedScript>>,   // ONSIGNAL signal
    pub on_started: Option<Arc<ParsedScript>>,  // ONSTARTED signal
}

impl EventHandler for SoundEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONFINISHED" => self.on_finished.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONRESUMED" => self.on_resumed.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            "ONSTARTED" => self.on_started.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    parent: Arc<CnvObject>,

    state: RefCell<SoundState>,
    event_handlers: SoundEventHandlers,

    should_flush_after_played: bool,
    should_preload: bool,
}

impl Sound {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: SoundProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(SoundState {
                music_volume: 1f32,
                ..Default::default()
            }),
            event_handlers: SoundEventHandlers {
                on_done: props.on_done,
                on_finished: props.on_finished,
                on_init: props.on_init,
                on_resumed: props.on_resumed,
                on_signal: props.on_signal,
                on_started: props.on_started,
            },
            should_flush_after_played: props.flush_after_played.unwrap_or_default(),
            should_preload: props.preload.unwrap_or_default(),
        }
    }

    // custom

    pub fn get_sound_to_play(&self) -> RunnerResult<Option<SoundData>> {
        let state = self.state.borrow();
        if !state.is_playing {
            return Ok(None);
        }
        let SoundFileData::Loaded(loaded_data) = &state.file_data else {
            return Ok(None);
        };
        Ok(Some(loaded_data.sound.clone()))
    }
}

impl CnvType for Sound {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SOUND"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("ISPLAYING") => self
                .state
                .borrow()
                .is_playing()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("LOAD") => self.state.borrow_mut().load().map(|_| None),
            CallableIdentifier::Method("PAUSE") => self.state.borrow_mut().pause().map(|_| None),
            CallableIdentifier::Method("PLAY") => {
                self.state.borrow_mut().play(context).map(|_| None)
            }
            CallableIdentifier::Method("RESUME") => self.state.borrow_mut().resume().map(|_| None),
            CallableIdentifier::Method("SETFREQ") => {
                self.state.borrow_mut().set_freq().map(|_| None)
            }
            CallableIdentifier::Method("SETPAN") => self.state.borrow_mut().set_pan().map(|_| None),
            CallableIdentifier::Method("SETVOLUME") => {
                self.state.borrow_mut().set_volume().map(|_| None)
            }
            CallableIdentifier::Method("STOP") => self.state.borrow_mut().stop().map(|_| None),
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => Err(RunnerError::InvalidCallable {
                object_name: self.parent.name.clone(),
                callable: ident.to_owned(),
            }),
        }
    }

    fn new_content(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_finished = properties
            .remove("ONFINISHED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_resumed = properties
            .remove("ONRESUMED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_started = properties
            .remove("ONSTARTED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::Sound(Self::from_initial_properties(
            parent,
            SoundProperties {
                filename,
                flush_after_played,
                preload,
                on_done,
                on_finished,
                on_init,
                on_resumed,
                on_signal,
                on_started,
            },
        )))
    }
}

impl Initable for Sound {
    fn initialize(&mut self, context: RunnerContext) -> RunnerResult<()> {
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                    arguments: Vec::new(),
                })
            });
        Ok(())
    }
}

impl SoundState {
    pub fn is_playing(&self) -> RunnerResult<bool> {
        // ISPLAYING
        todo!()
    }

    pub fn load(&mut self) -> RunnerResult<()> {
        // LOAD
        todo!()
    }

    pub fn pause(&mut self) -> RunnerResult<()> {
        // PAUSE
        todo!()
    }

    pub fn play(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // PLAY
        self.is_playing = true;
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundStarted(self.file_data.clone()))
            });
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONSTARTED").to_owned(),
                    arguments: Vec::new(),
                })
            });
        // FIXME: short-circuiting
        self.is_playing = false;
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundStopped(self.file_data.clone()))
            });
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONFINISHED").to_owned(),
                    arguments: Vec::new(),
                })
            });
        Ok(())
    }

    pub fn resume(&mut self) -> RunnerResult<()> {
        // RESUME
        todo!()
    }

    pub fn set_freq(&mut self) -> RunnerResult<()> {
        // SETFREQ
        todo!()
    }

    pub fn set_pan(&mut self) -> RunnerResult<()> {
        // SETPAN
        todo!()
    }

    pub fn set_volume(&mut self) -> RunnerResult<()> {
        // SETVOLUME
        todo!()
    }

    pub fn stop(&mut self) -> RunnerResult<()> {
        // STOP
        todo!()
    }
}
