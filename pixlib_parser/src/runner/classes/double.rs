use core::f64;
use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_bool, parse_event_handler, parse_f64};

use crate::{
    common::DroppableRefMut,
    parser::ast::ParsedScript,
    runner::{InternalEvent, RunnerError},
};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct DoubleVarProperties {
    // DOUBLE
    pub default: Option<f64>,    // DEFAULT
    pub netnotify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,    // TOINI
    pub value: Option<f64>,      // VALUE

    pub on_brutal_changed: HashMap<String, Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: HashMap<String, Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,                    // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,                    // ONINIT signal
    pub on_net_changed: HashMap<String, Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct DoubleVarState {
    // initialized from properties
    pub default_value: f64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct DoubleVarEventHandlers {
    pub on_brutal_changed: HashMap<String, Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: HashMap<String, Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,                    // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,                    // ONINIT signal
    pub on_net_changed: HashMap<String, Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>,         // ONSIGNAL signal
}

impl EventHandler for DoubleVarEventHandlers {
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
pub struct DoubleVar {
    parent: Arc<CnvObject>,

    state: RefCell<DoubleVarState>,
    event_handlers: DoubleVarEventHandlers,

    should_notify_on_net_changed: bool,
    should_be_stored_to_ini: bool,
}

impl DoubleVar {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: DoubleVarProperties) -> Self {
        let value = props.value.unwrap_or_default();
        Self {
            parent,
            state: RefCell::new(DoubleVarState {
                value,
                default_value: props.default.unwrap_or(value),
            }),
            event_handlers: DoubleVarEventHandlers {
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

    pub fn get(&self) -> RunnerResult<f64> {
        self.state.borrow().get()
    }
}

impl CnvType for DoubleVar {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "DOUBLE"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("ADD") => self
                .state
                .borrow_mut()
                .add(context, arguments[0].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("ARCTAN") => self
                .state
                .borrow_mut()
                .arc_tan(context, arguments[0].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("ARCTANEX") => self
                .state
                .borrow_mut()
                .arc_tan_ex(
                    context,
                    arguments[0].to_dbl(),
                    arguments[1].to_dbl(),
                    arguments.get(2).map(|v| v.to_int()),
                )
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("CLAMP") => self
                .state
                .borrow_mut()
                .clamp(context, arguments[0].to_dbl(), arguments[1].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("CLEAR") => {
                self.state.borrow_mut().clear(context).map(|_| None)
            }
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file(context).map(|_| None)
            }
            CallableIdentifier::Method("COSINUS") => self
                .state
                .borrow_mut()
                .cosinus(context, arguments[0].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("DEC") => self.state.borrow_mut().dec(context).map(|_| None),
            CallableIdentifier::Method("DIV") => self
                .state
                .borrow_mut()
                .div(context, arguments[0].to_dbl())
                .map(|_| None),
            CallableIdentifier::Method("GET") => {
                self.state.borrow().get().map(|v| Some(CnvValue::Double(v)))
            }
            CallableIdentifier::Method("INC") => self.state.borrow_mut().inc(context).map(|_| None),
            CallableIdentifier::Method("LENGTH") => self
                .state
                .borrow_mut()
                .length(context, arguments[0].to_dbl(), arguments[1].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("LOG") => self
                .state
                .borrow_mut()
                .log(context, arguments[0].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("MAXA") => {
                if arguments.is_empty() {
                    return Err(RunnerError::TooFewArguments {
                        expected_min: 1,
                        actual: 0,
                    });
                }
                self.state
                    .borrow_mut()
                    .max_a(context, arguments.iter().map(|v| v.to_dbl()))
                    .map(|v| Some(CnvValue::Double(v)))
            }
            CallableIdentifier::Method("MINA") => {
                if arguments.is_empty() {
                    return Err(RunnerError::TooFewArguments {
                        expected_min: 1,
                        actual: 0,
                    });
                }
                self.state
                    .borrow_mut()
                    .min_a(context, arguments.iter().map(|v| v.to_dbl()))
                    .map(|v| Some(CnvValue::Double(v)))
            }
            CallableIdentifier::Method("MOD") => self
                .state
                .borrow_mut()
                .modulus(context, arguments[0].to_int())
                .map(|_| None),
            CallableIdentifier::Method("MUL") => self
                .state
                .borrow_mut()
                .mul(context, arguments[0].to_dbl())
                .map(|_| None),
            CallableIdentifier::Method("POWER") => self
                .state
                .borrow_mut()
                .power(context, arguments[0].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("RANDOM") => {
                self.state.borrow_mut().random(context).map(|_| None)
            }
            CallableIdentifier::Method("RESETINI") => {
                self.state.borrow_mut().reset_ini(context).map(|_| None)
            }
            CallableIdentifier::Method("ROUND") => self
                .state
                .borrow_mut()
                .round(context)
                .map(|v| Some(CnvValue::Integer(v))),
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(context, arguments[0].to_dbl())
                .map(|_| None),
            CallableIdentifier::Method("SETDEFAULT") => self
                .state
                .borrow_mut()
                .set_default(context, arguments[0].to_dbl())
                .map(|_| None),
            CallableIdentifier::Method("SGN") => self
                .state
                .borrow()
                .sgn()
                .map(|v| Some(CnvValue::Integer(v))),
            CallableIdentifier::Method("SINUS") => self
                .state
                .borrow_mut()
                .sinus(context, arguments[0].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("SQRT") => self
                .state
                .borrow_mut()
                .sqrt(context)
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("SUB") => self
                .state
                .borrow_mut()
                .sub(context, arguments[0].to_dbl())
                .map(|v| Some(CnvValue::Double(v))),
            CallableIdentifier::Method("SWITCH") => self
                .state
                .borrow_mut()
                .switch(context, arguments[0].to_dbl(), arguments[1].to_dbl())
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
            }),
        }
    }

    fn new_content(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_f64)
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
            .map(parse_f64)
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
        Ok(CnvContent::Double(DoubleVar::from_initial_properties(
            parent,
            DoubleVarProperties {
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

impl Initable for DoubleVar {
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

const RADIANS_TO_DEGREES: f64 = 180f64 / f64::consts::PI;
const DEGREES_TO_RADIANS: f64 = f64::consts::PI / 180f64;

impl DoubleVarState {
    pub fn add(&mut self, context: RunnerContext, operand: f64) -> RunnerResult<f64> {
        // ADD
        self.change_value(context, self.value + operand);
        Ok(self.value)
    }

    pub fn arc_tan(&mut self, context: RunnerContext, tangent: f64) -> RunnerResult<f64> {
        // ARCTAN
        self.change_value(context, tangent.atan() * RADIANS_TO_DEGREES);
        Ok(self.value)
    }

    pub fn arc_tan_ex(
        &mut self,
        context: RunnerContext,
        y: f64,
        x: f64,
        summand: Option<i32>,
    ) -> RunnerResult<f64> {
        // ARCTANEX
        let mut value = (libm::atan2(y, x) + f64::consts::PI) * RADIANS_TO_DEGREES;
        if let Some(summand) = summand {
            value = value.trunc() + summand as f64;
        }
        self.change_value(context, value);
        Ok(self.value)
    }

    pub fn clamp(&mut self, context: RunnerContext, min: f64, max: f64) -> RunnerResult<f64> {
        // CLAMP
        self.change_value(context, self.value.clamp(min, max));
        Ok(self.value)
    }

    pub fn clear(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // CLEAR
        self.change_value(context, 0f64);
        Ok(())
    }

    pub fn copy_file(&mut self, _context: RunnerContext) -> RunnerResult<bool> {
        // COPYFILE
        todo!()
    }

    pub fn cosinus(&mut self, context: RunnerContext, angle_degrees: f64) -> RunnerResult<f64> {
        // COSINUS
        self.change_value(context, (angle_degrees * DEGREES_TO_RADIANS).cos());
        Ok(self.value)
    }

    pub fn dec(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // DEC
        self.change_value(context, self.value - 1f64);
        Ok(())
    }

    pub fn div(&mut self, context: RunnerContext, divisor: f64) -> RunnerResult<()> {
        // DIV
        self.change_value(context, self.value / divisor);
        Ok(())
    }

    pub fn get(&self) -> RunnerResult<f64> {
        // GET
        Ok(self.value)
    }

    pub fn inc(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // INC
        self.change_value(context, self.value + 1f64);
        Ok(())
    }

    pub fn length(&mut self, context: RunnerContext, x: f64, y: f64) -> RunnerResult<f64> {
        // LENGTH
        self.change_value(context, (x.powi(2) + y.powi(2)).sqrt());
        Ok(self.value)
    }

    pub fn log(&mut self, context: RunnerContext, operand: f64) -> RunnerResult<f64> {
        // LOG
        self.change_value(context, operand.ln());
        Ok(self.value)
    }

    pub fn max_a(
        &mut self,
        context: RunnerContext,
        arguments: impl Iterator<Item = f64>,
    ) -> RunnerResult<f64> {
        // MAXA
        self.change_value(context, arguments.reduce(f64::max).unwrap());
        Ok(self.value)
    }

    pub fn min_a(
        &mut self,
        context: RunnerContext,
        arguments: impl Iterator<Item = f64>,
    ) -> RunnerResult<f64> {
        // MINA
        self.change_value(context, arguments.reduce(f64::min).unwrap());
        Ok(self.value)
    }

    pub fn modulus(&mut self, context: RunnerContext, divisor: i32) -> RunnerResult<()> {
        // MOD
        self.change_value(context, (self.value as i32 % divisor) as f64);
        Ok(())
    }

    pub fn mul(&mut self, context: RunnerContext, operand: f64) -> RunnerResult<()> {
        // MUL
        self.change_value(context, self.value * operand);
        Ok(())
    }

    pub fn power(&mut self, context: RunnerContext, exponent: f64) -> RunnerResult<f64> {
        // POWER
        self.change_value(context, self.value.powf(exponent));
        Ok(self.value)
    }

    pub fn random(&mut self, _context: RunnerContext) -> RunnerResult<i32> {
        // RANDOM
        todo!()
    }

    pub fn reset_ini(&mut self, _context: RunnerContext) -> RunnerResult<()> {
        // RESETINI
        todo!()
    }

    pub fn round(&mut self, context: RunnerContext) -> RunnerResult<i32> {
        // ROUND
        self.change_value(context, self.value.round());
        Ok(self.value as i32)
    }

    pub fn set(&mut self, context: RunnerContext, value: f64) -> RunnerResult<()> {
        // SET
        self.change_value(context, value);
        Ok(())
    }

    pub fn set_default(&mut self, _context: RunnerContext, default_value: f64) -> RunnerResult<()> {
        // SETDEFAULT
        self.default_value = default_value;
        Ok(())
    }

    pub fn sgn(&self) -> RunnerResult<i32> {
        // SGN
        Ok(if self.value == 0.0 || self.value.is_nan() {
            0
        } else if self.value > 0.0 {
            1
        } else {
            -1
        })
    }

    pub fn sinus(&mut self, context: RunnerContext, angle_degrees: f64) -> RunnerResult<f64> {
        // SINUS
        self.change_value(context, (angle_degrees * DEGREES_TO_RADIANS).sin());
        Ok(self.value)
    }

    pub fn sqrt(&mut self, context: RunnerContext) -> RunnerResult<f64> {
        // SQRT
        self.change_value(context, self.value.sqrt());
        Ok(self.value)
    }

    pub fn sub(&mut self, context: RunnerContext, subtrahend: f64) -> RunnerResult<f64> {
        // SUB
        self.change_value(context, self.value - subtrahend);
        Ok(self.value)
    }

    pub fn switch(&mut self, context: RunnerContext, first: f64, second: f64) -> RunnerResult<()> {
        // SWITCH
        self.change_value(context, if self.value == first { second } else { first });
        Ok(())
    }

    // custom

    fn change_value(&mut self, context: RunnerContext, value: f64) {
        let changed = self.value != value;
        self.value = value;
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONBRUTALCHANGED").to_owned(),
                    arguments: vec![CnvValue::Double(self.value)],
                });
                if changed {
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifier::Event("ONCHANGED").to_owned(),
                        arguments: vec![CnvValue::Double(self.value)],
                    });
                }
            });
    }
}
