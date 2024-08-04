use std::any::Any;

use parsers::{discard_if_empty, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct SequenceInit {
    // SEQUENCE
    pub filename: Option<String>, // FILENAME

    pub on_done: Option<Arc<IgnorableProgram>>, // ONDONE signal
    pub on_finished: Option<Arc<IgnorableProgram>>, // ONFINISHED signal
    pub on_init: Option<Arc<IgnorableProgram>>, // ONINIT signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
    pub on_started: Option<Arc<IgnorableProgram>>, // ONSTARTED signal
}

#[derive(Debug, Clone)]
pub struct Sequence {
    // SEQUENCE
    parent: Arc<CnvObject>,
    initial_properties: SequenceInit,
}

impl Sequence {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: SequenceInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn get_event_name() {
        // GETEVENTNAME
        todo!()
    }

    pub fn get_playing() {
        // GETPLAYING
        todo!()
    }

    pub fn hide() {
        // HIDE
        todo!()
    }

    pub fn is_playing() {
        // ISPLAYING
        todo!()
    }

    pub fn pause() {
        // PAUSE
        todo!()
    }

    pub fn play() {
        // PLAY
        todo!()
    }

    pub fn resume() {
        // RESUME
        todo!()
    }

    pub fn set_freq() {
        // SETFREQ
        todo!()
    }

    pub fn set_pan() {
        // SETPAN
        todo!()
    }

    pub fn set_volume() {
        // SETVOLUME
        todo!()
    }

    pub fn show() {
        // SHOW
        todo!()
    }

    pub fn stop() {
        // STOP
        todo!()
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

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONDONE" | "ONFINISHED" | "ONINIT" | "ONSIGNAL" | "ONSTARTED"
        )
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "ONINIT" => self.initial_properties.on_init.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_finished = properties
            .remove("ONFINISHED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_started = properties
            .remove("ONSTARTED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            SequenceInit {
                filename,
                on_done,
                on_finished,
                on_init,
                on_signal,
                on_started,
            },
        ))
    }
}