use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct IntInit {
    // INTEGER
    pub default: Option<i32>,     // DEFAULT
    pub net_notify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,     // TOINI
    pub value: Option<i32>,       // VALUE

    pub on_brutal_changed: Option<Arc<IgnorableProgram>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<IgnorableProgram>>,        // ONCHANGED signal
    pub on_done: Option<Arc<IgnorableProgram>>,           // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<IgnorableProgram>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<IgnorableProgram>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Int {
    parent: Arc<CnvObject>,
    initial_properties: IntInit,
    value: i32,
}

impl Int {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: IntInit,
    ) -> Self {
        let value = initial_properties.value.unwrap_or(0);
        Self {
            parent,
            value,
            initial_properties,
        }
    }

    pub fn abs() {
        // ABS
        todo!()
    }

    pub fn add() {
        // ADD
        todo!()
    }

    pub fn and() {
        // AND
        todo!()
    }

    pub fn clamp() {
        // CLAMP
        todo!()
    }

    pub fn clear() {
        // CLEAR
        todo!()
    }

    pub fn copyfile() {
        // COPYFILE
        todo!()
    }

    pub fn dec() {
        // DEC
        todo!()
    }

    pub fn div() {
        // DIV
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

    pub fn modulus() {
        // MOD
        todo!()
    }

    pub fn mul() {
        // MUL
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

    pub fn power() {
        // POWER
        todo!()
    }

    pub fn random() {
        // RANDOM
        todo!()
    }

    pub fn resetini() {
        // RESETINI
        todo!()
    }

    pub fn set() {
        // SET
        todo!()
    }

    pub fn setdefault() {
        // SETDEFAULT
        todo!()
    }

    pub fn sub() {
        // SUB
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

impl CnvType for Int {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "INTEGER"
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
        name: CallableIdentifier,
        arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("SET") => {
                assert!(arguments.len() == 1);
                self.value = arguments[0].to_integer();
                Ok(None)
            }
            CallableIdentifier::Method("GET") => Ok(Some(CnvValue::Integer(self.value))),
            _ => todo!(),
        }
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
            .map(parse_i32)
            .transpose()?;
        let net_notify = properties
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
            .map(parse_i32)
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
            IntInit {
                default,
                net_notify,
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
