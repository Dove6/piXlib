use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_bool, parse_event_handler};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct BoolVarProperties {
    // BOOL
    pub default: Option<bool>,   // DEFAULT
    pub netnotify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,    // TOINI
    pub value: Option<bool>,     // VALUE

    pub on_brutal_changed: HashMap<String, Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: HashMap<String, Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,                    // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,                    // ONINIT signal
    pub on_net_changed: HashMap<String, Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct BoolVarState {
    // initialized from properties
    pub default_value: bool,
    pub value: i32,
}

#[derive(Debug, Clone)]
struct BoolVarEventHandlers {
    pub on_brutal_changed: HashMap<String, Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: HashMap<String, Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,                    // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,                    // ONINIT signal
    pub on_net_changed: HashMap<String, Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>,         // ONSIGNAL signal
}

impl EventHandler for BoolVarEventHandlers {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONBRUTALCHANGED" => argument
                .and_then(|a| self.on_brutal_changed.get(a))
                .or(self.on_brutal_changed.get("")),
            "ONCHANGED" => argument
                .and_then(|a| self.on_changed.get(a))
                .or(self.on_changed.get("")),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONNETCHANGED" => argument
                .and_then(|a| self.on_net_changed.get(a))
                .or(self.on_net_changed.get("")),
            "ONSIGNAL" => argument
                .and_then(|a| self.on_signal.get(a))
                .or(self.on_signal.get("")),
            _ => None,
        }
    }
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
                value: if value { 1 } else { 0 },
                default_value: props.default.unwrap_or(value),
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

    pub fn get(&self) -> anyhow::Result<bool> {
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("AND") => self
                .state
                .borrow_mut()
                .and(context, arguments[0].to_int())
                .map(|_| None),
            CallableIdentifier::Method("CLEAR") => {
                self.state.borrow_mut().clear(context).map(|_| None)
            }
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file(context).map(|_| None)
            }
            CallableIdentifier::Method("DEC") => self.state.borrow_mut().dec(context).map(|_| None),
            CallableIdentifier::Method("GET") => {
                self.state.borrow().get().map(|v| Some(CnvValue::Bool(v)))
            }
            CallableIdentifier::Method("INC") => self.state.borrow_mut().inc(context).map(|_| None),
            CallableIdentifier::Method("NOT") => self.state.borrow_mut().not(context).map(|_| None),
            CallableIdentifier::Method("OR") => self
                .state
                .borrow_mut()
                .or(context, arguments[0].to_int())
                .map(|_| None),
            CallableIdentifier::Method("RANDOM") => {
                self.state.borrow_mut().random(context).map(|_| None)
            }
            CallableIdentifier::Method("RESETINI") => {
                self.state.borrow_mut().reset_ini(context).map(|_| None)
            }
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(context, arguments[0].to_bool())
                .map(|_| None),
            CallableIdentifier::Method("SETDEFAULT") => self
                .state
                .borrow_mut()
                .set_default(context, arguments[0].to_bool())
                .map(|_| None),
            CallableIdentifier::Method("SWITCH") => {
                self.state.borrow_mut().switch(context).map(|_| None)
            }
            CallableIdentifier::Method("XOR") => self
                .state
                .borrow_mut()
                .xor(context, arguments[0].to_int())
                .map(|_| None),
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
            }
            .into()),
        }
    }

    fn new_content(
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
        let mut on_brutal_changed = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONBRUTALCHANGED" {
                on_brutal_changed.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONBRUTALCHANGED^") {
                on_brutal_changed
                    .insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_changed = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONCHANGED" {
                on_changed.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONCHANGED^") {
                on_changed.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_net_changed = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONNETCHANGED" {
                on_net_changed.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONNETCHANGED^") {
                on_net_changed.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_signal = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSIGNAL" {
                on_signal.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONSIGNAL^") {
                on_signal.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
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

impl Initable for BoolVar {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
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

const I32_WITH_CLEARED_U8: i32 = 0xffffff00u32 as i32;

impl BoolVarState {
    pub fn and(&mut self, context: RunnerContext, operand: i32) -> anyhow::Result<i32> {
        // AND
        self.change_value(context, self.value & operand);
        Ok(self.value)
    }

    pub fn clear(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // CLEAR
        self.change_value(context, self.value & I32_WITH_CLEARED_U8);
        Ok(())
    }

    pub fn copy_file(&mut self, _context: RunnerContext) -> anyhow::Result<()> {
        // COPYFILE
        todo!()
    }

    pub fn dec(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // DEC
        let value = match self.value as u8 {
            0 => (self.value & I32_WITH_CLEARED_U8) | 0x01,
            1 => self.value & I32_WITH_CLEARED_U8,
            _ => {
                self.value = (self.value & I32_WITH_CLEARED_U8) | 0x01; // a strange case - ONCHANGED needs to be emitted with the same value
                self.value & I32_WITH_CLEARED_U8
            }
        };
        self.change_value(context, value);
        Ok(())
    }

    pub fn get(&self) -> anyhow::Result<bool> {
        // GET
        Ok(self.value as u8 == 1)
    }

    pub fn inc(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // INC
        self.dec(context)
    }

    pub fn not(&mut self, context: RunnerContext) -> anyhow::Result<i32> {
        // NOT
        self.change_value(context, !self.value);
        Ok(self.value)
    }

    pub fn or(&mut self, context: RunnerContext, operand: i32) -> anyhow::Result<i32> {
        // OR
        self.change_value(context, self.value | operand);
        Ok(self.value)
    }

    pub fn random(&mut self, _context: RunnerContext) -> anyhow::Result<()> {
        // RANDOM
        todo!()
    }

    pub fn reset_ini(&mut self, _context: RunnerContext) -> anyhow::Result<()> {
        // RESETINI
        todo!()
    }

    pub fn set(&mut self, context: RunnerContext, value: bool) -> anyhow::Result<()> {
        // SET
        self.change_value(
            context,
            (self.value & I32_WITH_CLEARED_U8) | if value { 0x01 } else { 0x00 },
        );
        Ok(())
    }

    pub fn set_default(
        &mut self,
        _context: RunnerContext,
        default_value: bool,
    ) -> anyhow::Result<()> {
        // SETDEFAULT
        self.default_value = default_value;
        Ok(())
    }

    pub fn switch(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // SWITCH
        self.dec(context)
    }

    pub fn xor(&mut self, context: RunnerContext, operand: i32) -> anyhow::Result<i32> {
        // XOR
        self.change_value(context, self.value ^ operand);
        Ok(self.value)
    }

    // custom

    fn change_value(&mut self, context: RunnerContext, value: i32) {
        let old_value = self.get().unwrap();
        self.value = value;
        let value = self.get().unwrap();
        let changed = value != old_value;
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONBRUTALCHANGED").to_owned(),
                    arguments: vec![CnvValue::Bool(value)],
                });
                if changed {
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifier::Event("ONCHANGED").to_owned(),
                        arguments: vec![CnvValue::Bool(value)],
                    });
                }
            });
    }
}
