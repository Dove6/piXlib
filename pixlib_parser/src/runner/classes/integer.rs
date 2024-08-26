use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_bool, parse_event_handler, parse_i32};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct IntegerVarProperties {
    // INTEGER
    pub default: Option<i32>,     // DEFAULT
    pub net_notify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,     // TOINI
    pub value: Option<i32>,       // VALUE

    pub on_brutal_changed: HashMap<String, Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: HashMap<String, Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,                    // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,                    // ONINIT signal
    pub on_net_changed: HashMap<String, Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct IntegerVarState {
    // initialized from properties
    pub default_value: i32,
    pub value: i32,
}

#[derive(Debug, Clone)]
pub struct IntegerVarEventHandlers {
    pub on_brutal_changed: HashMap<String, Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: HashMap<String, Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,                    // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,                    // ONINIT signal
    pub on_net_changed: HashMap<String, Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>,         // ONSIGNAL signal
}

impl EventHandler for IntegerVarEventHandlers {
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
pub struct IntegerVar {
    parent: Arc<CnvObject>,

    state: RefCell<IntegerVarState>,
    event_handlers: IntegerVarEventHandlers,

    should_notify_on_net_changed: bool,
    should_be_stored_to_ini: bool,
}

impl IntegerVar {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: IntegerVarProperties) -> Self {
        let value = props.value.unwrap_or_default();
        Self {
            parent,
            state: RefCell::new(IntegerVarState {
                value,
                default_value: props.default.unwrap_or(value),
            }),
            event_handlers: IntegerVarEventHandlers {
                on_brutal_changed: props.on_brutal_changed,
                on_changed: props.on_changed,
                on_done: props.on_done,
                on_init: props.on_init,
                on_net_changed: props.on_net_changed,
                on_signal: props.on_signal,
            },
            should_notify_on_net_changed: props.net_notify.unwrap_or_default(),
            should_be_stored_to_ini: props.to_ini.unwrap_or_default(),
        }
    }

    pub fn get(&self) -> anyhow::Result<i32> {
        self.state.borrow().get(RunnerContext::new_minimal(
            &self.parent.parent.runner,
            &self.parent,
        ))
    }
}

impl CnvType for IntegerVar {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "INTEGER"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("ABS") => {
                self.state.borrow_mut().abs(context).map(CnvValue::Integer)
            }
            CallableIdentifier::Method("ADD") => self
                .state
                .borrow_mut()
                .add(context, arguments[0].to_int())
                .map(CnvValue::Integer),
            CallableIdentifier::Method("AND") => self
                .state
                .borrow_mut()
                .and(context, arguments[0].to_int())
                .map(CnvValue::Integer),
            CallableIdentifier::Method("CLAMP") => self
                .state
                .borrow_mut()
                .clamp(context, arguments[0].to_int(), arguments[1].to_int())
                .map(CnvValue::Integer),
            CallableIdentifier::Method("CLEAR") => self
                .state
                .borrow_mut()
                .clear(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("COPYFILE") => self
                .state
                .borrow_mut()
                .copy_file(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("DEC") => {
                self.state.borrow_mut().dec(context).map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DIV") => self
                .state
                .borrow_mut()
                .div(context, arguments[0].to_int())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GET") => {
                self.state.borrow().get(context).map(CnvValue::Integer)
            }
            CallableIdentifier::Method("INC") => {
                self.state.borrow_mut().inc(context).map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MOD") => self
                .state
                .borrow_mut()
                .modulus(context, arguments[0].to_int())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("MUL") => self
                .state
                .borrow_mut()
                .mul(context, arguments[0].to_int())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("NOT") => {
                self.state.borrow_mut().not(context).map(CnvValue::Integer)
            }
            CallableIdentifier::Method("OR") => self
                .state
                .borrow_mut()
                .or(context, arguments[0].to_int())
                .map(CnvValue::Integer),
            CallableIdentifier::Method("POWER") => self
                .state
                .borrow_mut()
                .power(context, arguments[0].to_int())
                .map(CnvValue::Integer),
            CallableIdentifier::Method("RANDOM") => self
                .state
                .borrow_mut()
                .random(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("RESETINI") => self
                .state
                .borrow_mut()
                .reset_ini(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(context, arguments[0].to_int())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETDEFAULT") => self
                .state
                .borrow_mut()
                .set_default(context, arguments[0].to_int())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SUB") => self
                .state
                .borrow_mut()
                .sub(context, arguments[0].to_int())
                .map(CnvValue::Integer),
            CallableIdentifier::Method("SWITCH") => self
                .state
                .borrow_mut()
                .switch(context, arguments[0].to_int(), arguments[1].to_int())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("XOR") => self
                .state
                .borrow_mut()
                .xor(context, arguments[0].to_int())
                .map(CnvValue::Integer),
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context).map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
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
        Ok(CnvContent::Integer(Self::from_initial_properties(
            parent,
            IntegerVarProperties {
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
        )))
    }
}

impl Initable for IntegerVar {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(Vec::new()),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                })
            });
        Ok(())
    }
}

impl IntegerVarState {
    pub fn abs(&mut self, context: RunnerContext) -> anyhow::Result<i32> {
        // ABS
        self.change_value(context, self.value.abs());
        Ok(self.value)
    }

    pub fn add(&mut self, context: RunnerContext, operand: i32) -> anyhow::Result<i32> {
        // ADD
        self.change_value(context, self.value + operand);
        Ok(self.value)
    }

    pub fn and(&mut self, context: RunnerContext, operand: i32) -> anyhow::Result<i32> {
        // AND
        self.change_value(context, self.value & operand);
        Ok(self.value)
    }

    pub fn clamp(&mut self, context: RunnerContext, min: i32, max: i32) -> anyhow::Result<i32> {
        // CLAMP
        self.change_value(context, self.value.clamp(min, max));
        Ok(self.value)
    }

    pub fn clear(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // CLEAR
        self.change_value(context, 0);
        Ok(())
    }

    pub fn copy_file(&mut self, _context: RunnerContext) -> anyhow::Result<()> {
        // COPYFILE
        todo!()
    }

    pub fn dec(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // DEC
        self.change_value(context, self.value - 1);
        Ok(())
    }

    pub fn div(&mut self, context: RunnerContext, divisor: i32) -> anyhow::Result<()> {
        // DIV
        self.change_value(context, self.value / divisor);
        Ok(())
    }

    pub fn get(&self, _context: RunnerContext) -> anyhow::Result<i32> {
        // GET
        Ok(self.value)
    }

    pub fn inc(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // INC
        self.change_value(context, self.value + 1);
        Ok(())
    }

    pub fn modulus(&mut self, context: RunnerContext, divisor: i32) -> anyhow::Result<()> {
        // MOD
        self.change_value(context, self.value % divisor);
        Ok(())
    }

    pub fn mul(&mut self, context: RunnerContext, operand: i32) -> anyhow::Result<()> {
        // MUL
        self.change_value(context, self.value * operand);
        Ok(())
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

    pub fn power(&mut self, context: RunnerContext, exponent: i32) -> anyhow::Result<i32> {
        // POWER
        self.change_value(
            context,
            if exponent < 0 {
                0i32
            } else {
                self.value.pow(exponent as u32)
            },
        );
        Ok(self.value)
    }

    pub fn random(&mut self, _context: RunnerContext) -> anyhow::Result<()> {
        // RANDOM
        todo!()
    }

    pub fn reset_ini(&mut self, _context: RunnerContext) -> anyhow::Result<()> {
        // RESETINI
        eprintln!("Skipping INTEGER^RESETINI() call");
        Ok(())
    }

    pub fn set(&mut self, context: RunnerContext, value: i32) -> anyhow::Result<()> {
        // SET
        self.change_value(context, value);
        Ok(())
    }

    pub fn set_default(
        &mut self,
        _context: RunnerContext,
        default_value: i32,
    ) -> anyhow::Result<()> {
        // SETDEFAULT
        self.default_value = default_value;
        Ok(())
    }

    pub fn sub(&mut self, context: RunnerContext, subtrahend: i32) -> anyhow::Result<i32> {
        // SUB
        self.change_value(context, self.value - subtrahend);
        Ok(self.value)
    }

    pub fn switch(
        &mut self,
        context: RunnerContext,
        first: i32,
        second: i32,
    ) -> anyhow::Result<()> {
        // SWITCH
        self.change_value(context, if self.value == first { second } else { first });
        Ok(())
    }

    pub fn xor(&mut self, context: RunnerContext, operand: i32) -> anyhow::Result<i32> {
        // XOR
        self.change_value(context, self.value ^ operand);
        Ok(self.value)
    }

    // custom

    fn change_value(&mut self, context: RunnerContext, value: i32) {
        let changed = self.value != value;
        self.value = value;
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context
                        .clone()
                        .with_arguments(vec![CnvValue::Integer(self.value)]),
                    callable: CallableIdentifier::Event("ONBRUTALCHANGED").to_owned(),
                });
                if changed {
                    events.push_back(InternalEvent {
                        context: context
                            .clone()
                            .with_arguments(vec![CnvValue::Integer(self.value)]),
                        callable: CallableIdentifier::Event("ONCHANGED").to_owned(),
                    });
                }
            });
    }
}
