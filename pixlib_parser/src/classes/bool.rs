use std::any::Any;

use parsers::{discard_if_empty, parse_bool, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct BoolInit {
    // BOOL
    pub default: Option<bool>,   // DEFAULT
    pub netnotify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,    // TOINI
    pub value: Option<bool>,     // VALUE

    pub on_brutal_changed: Option<Arc<IgnorableProgram>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<IgnorableProgram>>,        // ONCHANGED signal
    pub on_done: Option<Arc<IgnorableProgram>>,           // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<IgnorableProgram>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<IgnorableProgram>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Bool {
    parent: Arc<CnvObject>,
    initial_properties: BoolInit,
    value: bool,
}

impl Bool {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: BoolInit) -> Self {
        let value = initial_properties.value.unwrap_or(false);
        Self {
            parent,
            value,
            initial_properties,
        }
    }

    pub fn and() {
        // AND
        todo!()
    }

    pub fn clear() {
        // CLEAR
        todo!()
    }

    pub fn copy_file() {
        // COPYFILE
        todo!()
    }

    pub fn dec() {
        // DEC
        todo!()
    }

    pub fn get() {
        // GET
        todo!()
    }

    pub fn inc() {
        // INC
        todo!()
    }

    pub fn not() {
        // NOT
        todo!()
    }

    pub fn or() {
        // OR
        todo!()
    }

    pub fn random() {
        // RANDOM
        todo!()
    }

    pub fn reset_ini() {
        // RESETINI
        todo!()
    }

    pub fn set() {
        // SET
        todo!()
    }

    pub fn set_default() {
        // SETDEFAULT
        todo!()
    }

    pub fn switch() {
        // SWITCH
        todo!()
    }

    pub fn xor() {
        // XOR
        todo!()
    }
}

impl CnvType for Bool {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BOOL"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties
            .remove("VALUE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let on_brutal_changed = properties
            .remove("ONBRUTALCHANGED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_changed = properties
            .remove("ONCHANGED")
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
        let on_net_changed = properties
            .remove("ONNETCHANGED")
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
            BoolInit {
                default,
                netnotify,
                to_ini,
                value,
                on_brutal_changed,
                on_changed,
                on_done,
                on_init,
                on_net_changed,
                on_signal,
            },
        ))
    }
}
