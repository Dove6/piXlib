use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_f64, parse_program};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct DoubleVarProperties {
    // DOUBLE
    pub default: Option<f64>,    // DEFAULT
    pub netnotify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,    // TOINI
    pub value: Option<f64>,      // VALUE

    pub on_brutal_changed: Option<Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct DoubleVarState {
    pub initialized: bool,

    // initialized from properties
    pub default_value: f64,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct DoubleVarEventHandlers {
    pub on_brutal_changed: Option<Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<ParsedScript>>,         // ONSIGNAL signal
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
                ..Default::default()
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
            CallableIdentifier::Method("ADD") => self.state.borrow_mut().add().map(|_| None),
            CallableIdentifier::Method("ARCTAN") => self.state.borrow_mut().arc_tan().map(|_| None),
            CallableIdentifier::Method("ARCTANEX") => {
                self.state.borrow_mut().arc_tan_ex().map(|_| None)
            }
            CallableIdentifier::Method("CLAMP") => self.state.borrow_mut().clamp().map(|_| None),
            CallableIdentifier::Method("CLEAR") => self.state.borrow_mut().clear().map(|_| None),
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file().map(|_| None)
            }
            CallableIdentifier::Method("COSINUS") => {
                self.state.borrow_mut().cosinus().map(|_| None)
            }
            CallableIdentifier::Method("DEC") => self.state.borrow_mut().dec().map(|_| None),
            CallableIdentifier::Method("DIV") => self.state.borrow_mut().div().map(|_| None),
            CallableIdentifier::Method("GET") => {
                self.state.borrow().get().map(|v| Some(CnvValue::Double(v)))
            }
            CallableIdentifier::Method("INC") => self.state.borrow_mut().inc().map(|_| None),
            CallableIdentifier::Method("LENGTH") => self.state.borrow_mut().length().map(|_| None),
            CallableIdentifier::Method("LOG") => self.state.borrow_mut().log().map(|_| None),
            CallableIdentifier::Method("MAXA") => self.state.borrow_mut().max_a().map(|_| None),
            CallableIdentifier::Method("MINA") => self.state.borrow_mut().min_a().map(|_| None),
            CallableIdentifier::Method("MOD") => self.state.borrow_mut().modulus().map(|_| None),
            CallableIdentifier::Method("MUL") => self.state.borrow_mut().mul().map(|_| None),
            CallableIdentifier::Method("POWER") => self.state.borrow_mut().power().map(|_| None),
            CallableIdentifier::Method("RANDOM") => self.state.borrow_mut().random().map(|_| None),
            CallableIdentifier::Method("RESETINI") => {
                self.state.borrow_mut().reset_ini().map(|_| None)
            }
            CallableIdentifier::Method("ROUND") => self.state.borrow_mut().round().map(|_| None),
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(self, arguments[0].to_double())
                .map(|_| None),
            CallableIdentifier::Method("SETDEFAULT") => {
                self.state.borrow_mut().set_default().map(|_| None)
            }
            CallableIdentifier::Method("SGN") => self.state.borrow_mut().sgn().map(|_| None),
            CallableIdentifier::Method("SINUS") => self.state.borrow_mut().sinus().map(|_| None),
            CallableIdentifier::Method("SQRT") => self.state.borrow_mut().sqrt().map(|_| None),
            CallableIdentifier::Method("SUB") => self.state.borrow_mut().sub().map(|_| None),
            CallableIdentifier::Method("SWITCH") => self.state.borrow_mut().switch().map(|_| None),
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

impl DoubleVarState {
    pub fn add(&mut self) -> RunnerResult<()> {
        // ADD
        todo!()
    }

    pub fn arc_tan(&mut self) -> RunnerResult<()> {
        // ARCTAN
        todo!()
    }

    pub fn arc_tan_ex(&mut self) -> RunnerResult<()> {
        // ARCTANEX
        todo!()
    }

    pub fn clamp(&mut self) -> RunnerResult<()> {
        // CLAMP
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

    pub fn cosinus(&mut self) -> RunnerResult<()> {
        // COSINUS
        todo!()
    }

    pub fn dec(&mut self) -> RunnerResult<()> {
        // DEC
        todo!()
    }

    pub fn div(&mut self) -> RunnerResult<()> {
        // DIV
        todo!()
    }

    pub fn get(&self) -> RunnerResult<f64> {
        // GET
        Ok(self.value)
    }

    pub fn inc(&mut self) -> RunnerResult<()> {
        // INC
        todo!()
    }

    pub fn length(&mut self) -> RunnerResult<()> {
        // LENGTH
        todo!()
    }

    pub fn log(&mut self) -> RunnerResult<()> {
        // LOG
        todo!()
    }

    pub fn max_a(&mut self) -> RunnerResult<()> {
        // MAXA
        todo!()
    }

    pub fn min_a(&mut self) -> RunnerResult<()> {
        // MINA
        todo!()
    }

    pub fn modulus(&mut self) -> RunnerResult<()> {
        // MOD
        todo!()
    }

    pub fn mul(&mut self) -> RunnerResult<()> {
        // MUL
        todo!()
    }

    pub fn power(&mut self) -> RunnerResult<()> {
        // POWER
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

    pub fn round(&mut self) -> RunnerResult<()> {
        // ROUND
        todo!()
    }

    pub fn set(&mut self, double: &DoubleVar, value: f64) -> RunnerResult<()> {
        // SET
        let changed_value = self.value != value;
        self.value = value;
        let context = RunnerContext {
            runner: Arc::clone(&double.parent.parent.runner),
            self_object: double.parent.name.clone(),
            current_object: double.parent.name.clone(),
        };
        if changed_value {
            double.call_method(
                CallableIdentifier::Event("ONCHANGED"),
                &vec![CnvValue::Double(self.value)],
                context.clone(),
            )?;
        }
        double.call_method(
            CallableIdentifier::Event("ONBRUTALCHANGED"),
            &vec![CnvValue::Double(self.value)],
            context,
        )?;
        Ok(())
    }

    pub fn set_default(&mut self) -> RunnerResult<()> {
        // SETDEFAULT
        todo!()
    }

    pub fn sgn(&mut self) -> RunnerResult<()> {
        // SGN
        todo!()
    }

    pub fn sinus(&mut self) -> RunnerResult<()> {
        // SINUS
        todo!()
    }

    pub fn sqrt(&mut self) -> RunnerResult<()> {
        // SQRT
        todo!()
    }

    pub fn sub(&mut self) -> RunnerResult<()> {
        // SUB
        todo!()
    }

    pub fn switch(&mut self) -> RunnerResult<()> {
        // SWITCH
        todo!()
    }
}
