use std::any::Any;

use parsers::{discard_if_empty, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct KeyboardInit {
    // KEYBOARD
    pub keyboard: Option<String>, // KEYBOARD

    pub on_char: Option<Arc<IgnorableProgram>>, // ONCHAR signal
    pub on_done: Option<Arc<IgnorableProgram>>, // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>, // ONINIT signal
    pub on_key_down: Option<Arc<IgnorableProgram>>, // ONKEYDOWN signal
    pub on_key_up: Option<Arc<IgnorableProgram>>, // ONKEYUP signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    parent: Arc<CnvObject>,
    initial_properties: KeyboardInit,
}

impl Keyboard {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: KeyboardInit,
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

    pub fn get_latest_key() {
        // GETLATESTKEY
        todo!()
    }

    pub fn get_latest_keys() {
        // GETLATESTKEYS
        todo!()
    }

    pub fn is_enabled() {
        // ISENABLED
        todo!()
    }

    pub fn is_key_down() {
        // ISKEYDOWN
        todo!()
    }

    pub fn set_auto_repeat() {
        // SETAUTOREPEAT
        todo!()
    }
}

impl CnvType for Keyboard {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "KEYBOARD"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONCHAR" | "ONDONE" | "ONINIT" | "ONKEYDOWN" | "ONKEYUP" | "ONSIGNAL"
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
        let keyboard = properties.remove("KEYBOARD").and_then(discard_if_empty);
        let on_char = properties
            .remove("ONCHAR")
            .and_then(discard_if_empty)
            .map(parse_program)
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
        let on_key_down = properties
            .remove("ONKEYDOWN")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_key_up = properties
            .remove("ONKEYUP")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            KeyboardInit {
                keyboard,
                on_char,
                on_done,
                on_init,
                on_key_down,
                on_key_up,
                on_signal,
            },
        ))
    }
}
