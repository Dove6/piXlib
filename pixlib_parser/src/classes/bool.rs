use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_program};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct BoolVarProperties {
    // BOOL
    pub default: Option<bool>,   // DEFAULT
    pub netnotify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,    // TOINI
    pub value: Option<bool>,     // VALUE

    pub on_brutal_changed: Option<Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct BoolVarState {
    pub initialized: bool,

    // initialized from properties
    pub default_value: bool,
    pub value: bool,
}

#[derive(Debug, Clone)]
struct BoolVarEventHandlers {
    pub on_brutal_changed: Option<Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct BoolVar {
    parent: Arc<CnvObject>,

    state: RefCell<BoolVarState>,
    event_handlers: BoolVarEventHandlers,

    should_notify_on_net_changed: bool,
    should_be_stored_to_ini: bool,
}

impl BoolVar {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: BoolVarProperties) -> Self {
        let value = props.value.unwrap_or_default();
        Self {
            parent,
            state: RefCell::new(BoolVarState {
                value,
                default_value: props.default.unwrap_or(value),
                ..Default::default()
            }),
            event_handlers: BoolVarEventHandlers {
                on_brutal_changed: props.on_brutal_changed,
                on_changed: props.on_changed,
                on_done: props.on_done,
                on_init: props.on_init,
                on_net_changed: props.on_net_changed,
                on_signal: props.on_signal,
            },
            should_notify_on_net_changed: props.netnotify.unwrap_or_default(),
            should_be_stored_to_ini: props.to_ini.unwrap_or_default(),
        }
    }

    pub fn get(&self) -> RunnerResult<bool> {
        self.state.borrow().get()
    }
}

impl CnvType for BoolVar {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BOOL"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONBRUTALCHANGED" | "ONCHANGED" | "ONDONE" | "ONINIT" | "ONNETCHANGED" | "ONSIGNAL"
        )
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("AND") => self.state.borrow_mut().and().map(|_| None),
            CallableIdentifier::Method("CLEAR") => self.state.borrow_mut().clear().map(|_| None),
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file().map(|_| None)
            }
            CallableIdentifier::Method("DEC") => self.state.borrow_mut().dec().map(|_| None),
            CallableIdentifier::Method("GET") => self
                .state
                .borrow()
                .get()
                .map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Method("INC") => self.state.borrow_mut().inc().map(|_| None),
            CallableIdentifier::Method("NOT") => self.state.borrow_mut().not().map(|_| None),
            CallableIdentifier::Method("OR") => self.state.borrow_mut().or().map(|_| None),
            CallableIdentifier::Method("RANDOM") => self.state.borrow_mut().random().map(|_| None),
            CallableIdentifier::Method("RESETINI") => {
                self.state.borrow_mut().reset_ini().map(|_| None)
            }
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(self, context, arguments[0].to_boolean())
                .map(|_| None),
            CallableIdentifier::Method("SETDEFAULT") => {
                self.state.borrow_mut().set_default().map(|_| None)
            }
            CallableIdentifier::Method("SWITCH") => self.state.borrow_mut().switch().map(|_| None),
            CallableIdentifier::Method("XOR") => self.state.borrow_mut().xor().map(|_| None),
            CallableIdentifier::Event("ONBRUTALCHANGED") => {
                if let Some(v) = self.event_handlers.on_brutal_changed.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONCHANGED") => {
                if let Some(v) = self.event_handlers.on_changed.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.event_handlers.on_done.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.event_handlers.on_init.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONNETCHANGED") => {
                if let Some(v) = self.event_handlers.on_net_changed.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONSIGNAL") => {
                if let Some(v) = self.event_handlers.on_signal.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
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
        Ok(CnvContent::Bool(Self::from_initial_properties(
            parent,
            BoolVarProperties {
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
        )))
    }
}

impl BoolVarState {
    pub fn and(&mut self) -> RunnerResult<()> {
        // AND
        todo!()
    }

    pub fn clear(&mut self) -> RunnerResult<()> {
        // CLEAR
        todo!()
    }

    pub fn copy_file(&mut self) -> RunnerResult<()> {
        // COPYFILE
        todo!()
    }

    pub fn dec(&mut self) -> RunnerResult<()> {
        // DEC
        todo!()
    }

    pub fn get(&self) -> RunnerResult<bool> {
        // GET
        Ok(self.value)
    }

    pub fn inc(&mut self) -> RunnerResult<()> {
        // INC
        todo!()
    }

    pub fn not(&mut self) -> RunnerResult<()> {
        // NOT
        todo!()
    }

    pub fn or(&mut self) -> RunnerResult<()> {
        // OR
        todo!()
    }

    pub fn random(&mut self) -> RunnerResult<()> {
        // RANDOM
        todo!()
    }

    pub fn reset_ini(&mut self) -> RunnerResult<()> {
        // RESETINI
        todo!()
    }

    pub fn set(
        &mut self,
        boolean: &BoolVar,
        context: RunnerContext,
        value: bool,
    ) -> RunnerResult<()> {
        // SET
        let changed_value = self.value != value;
        self.value = value;
        if changed_value {
            boolean.call_method(
                CallableIdentifier::Event("ONCHANGED"),
                &vec![CnvValue::Boolean(self.value)],
                context.clone(),
            )?;
        }
        boolean.call_method(
            CallableIdentifier::Event("ONBRUTALCHANGED"),
            &vec![CnvValue::Boolean(self.value)],
            context,
        )?;
        Ok(())
    }

    pub fn set_default(&mut self) -> RunnerResult<()> {
        // SETDEFAULT
        todo!()
    }

    pub fn switch(&mut self) -> RunnerResult<()> {
        // SWITCH
        todo!()
    }

    pub fn xor(&mut self) -> RunnerResult<()> {
        // XOR
        todo!()
    }
}
