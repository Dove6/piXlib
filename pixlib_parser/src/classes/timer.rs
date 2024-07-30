use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct TimerInit {
    // TIMER
    pub elapse: Option<i32>,   // ELAPSE
    pub enabled: Option<bool>, // ENABLED
    pub ticks: Option<i32>,    // TICKS

    pub on_done: Option<Arc<IgnorableProgram>>, // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>, // ONINIT signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
    pub on_tick: Option<Arc<IgnorableProgram>>, // ONTICK signal
}

#[derive(Debug, Clone)]
pub struct Timer {
    parent: Arc<RwLock<CnvObject>>,
    initial_properties: TimerInit,
}

impl Timer {
    pub fn from_initial_properties(
        parent: Arc<RwLock<CnvObject>>,
        initial_properties: TimerInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn disable() {
        // DISABLE
        todo!()
    }

    pub fn enable() {
        // ENABLE
        todo!()
    }

    pub fn get_ticks() {
        // GETTICKS
        todo!()
    }

    pub fn pause() {
        // PAUSE
        todo!()
    }

    pub fn reset() {
        // RESET
        todo!()
    }

    pub fn resume() {
        // RESUME
        todo!()
    }

    pub fn set() {
        // SET
        todo!()
    }

    pub fn set_elapse() {
        // SETELAPSE
        todo!()
    }
}

impl CnvType for Timer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "TIMER"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(name, "ONDONE" | "ONINIT" | "ONSIGNAL" | "ONTICK")
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

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<RwLock<CnvObject>>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let elapse = properties
            .remove("ELAPSE")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let enabled = properties
            .remove("ENABLED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let ticks = properties
            .remove("TICKS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
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
        let on_tick = properties
            .remove("ONTICK")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            TimerInit {
                elapse,
                enabled,
                ticks,
                on_done,
                on_init,
                on_signal,
                on_tick,
            },
        ))
    }
}
