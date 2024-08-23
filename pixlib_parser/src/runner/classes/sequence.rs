use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct SequenceProperties {
    // SEQUENCE
    pub filename: Option<String>, // FILENAME

    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_finished: Option<Arc<ParsedScript>>, // ONFINISHED signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_started: Option<Arc<ParsedScript>>, // ONSTARTED signal
}

#[derive(Debug, Clone, Default)]
struct SequenceState {
    pub initialized: bool,

    // initialized from properties
    pub file_data: SequenceFileData,

    // deduced from methods
    pub is_playing: bool,
    pub is_visible: bool,
    pub music_frequency: usize,
    pub music_volume: f32,
    pub music_pan: f32,
}

#[derive(Debug, Clone)]
pub struct SequenceEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>,     // ONDONE signal
    pub on_finished: Option<Arc<ParsedScript>>, // ONFINISHED signal
    pub on_init: Option<Arc<ParsedScript>>,     // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>,   // ONSIGNAL signal
    pub on_started: Option<Arc<ParsedScript>>,  // ONSTARTED signal
}

impl EventHandler for SequenceEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONFINISHED" => self.on_finished.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            "ONSTARTED" => self.on_started.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    // SEQUENCE
    parent: Arc<CnvObject>,

    state: RefCell<SequenceState>,
    event_handlers: SequenceEventHandlers,
}

impl Sequence {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: SequenceProperties) -> Self {
        let sequence = Self {
            parent,
            state: RefCell::new(SequenceState {
                is_visible: true,
                music_volume: 1f32,
                ..Default::default()
            }),
            event_handlers: SequenceEventHandlers {
                on_done: props.on_done,
                on_finished: props.on_finished,
                on_init: props.on_init,
                on_signal: props.on_signal,
                on_started: props.on_started,
            },
        };
        if let Some(filename) = props.filename {
            sequence.state.borrow_mut().file_data = SequenceFileData::NotLoaded(filename);
        }
        sequence
    }
}

impl CnvType for Sequence {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SEQUENCE"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("GETEVENTNAME") => self
                .state
                .borrow()
                .get_event_name()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETPLAYING") => self
                .state
                .borrow()
                .get_playing()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("HIDE") => self.state.borrow_mut().hide().map(|_| None),
            CallableIdentifier::Method("ISPLAYING") => self
                .state
                .borrow()
                .is_playing()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("PAUSE") => self.state.borrow_mut().pause().map(|_| None),
            CallableIdentifier::Method("PLAY") => self.state.borrow_mut().play().map(|_| None),
            CallableIdentifier::Method("RESUME") => self.state.borrow_mut().resume().map(|_| None),
            CallableIdentifier::Method("SETFREQ") => {
                self.state.borrow_mut().set_freq().map(|_| None)
            }
            CallableIdentifier::Method("SETPAN") => self.state.borrow_mut().set_pan().map(|_| None),
            CallableIdentifier::Method("SETVOLUME") => {
                self.state.borrow_mut().set_volume().map(|_| None)
            }
            CallableIdentifier::Method("SHOW") => self.state.borrow_mut().show().map(|_| None),
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
        Ok(CnvContent::Sequence(Self::from_initial_properties(
            parent,
            SequenceProperties {
                filename,
                on_done,
                on_finished,
                on_init,
                on_signal,
                on_started,
            },
        )))
    }
}

impl Initable for Sequence {
    fn initialize(&self, context: RunnerContext) -> RunnerResult<()> {
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

impl SequenceState {
    pub fn get_event_name(&self) -> RunnerResult<String> {
        // GETEVENTNAME
        todo!()
    }

    pub fn get_playing(&self) -> RunnerResult<String> {
        // GETPLAYING
        todo!()
    }

    pub fn hide(&mut self) -> RunnerResult<()> {
        // HIDE
        todo!()
    }

    pub fn is_playing(&self) -> RunnerResult<bool> {
        // ISPLAYING
        todo!()
    }

    pub fn pause(&mut self) -> RunnerResult<()> {
        // PAUSE
        todo!()
    }

    pub fn play(&mut self) -> RunnerResult<()> {
        // PLAY
        todo!()
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

    pub fn show(&mut self) -> RunnerResult<()> {
        // SHOW
        todo!()
    }

    pub fn stop(&mut self) -> RunnerResult<()> {
        // STOP
        todo!()
    }
}
