use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_f64, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct DblInit {
    // DOUBLE
    pub default: Option<f64>,    // DEFAULT
    pub netnotify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,    // TOINI
    pub value: Option<f64>,      // VALUE

    pub on_brutal_changed: Option<Arc<IgnorableProgram>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<IgnorableProgram>>,        // ONCHANGED signal
    pub on_done: Option<Arc<IgnorableProgram>>,           // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<IgnorableProgram>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<IgnorableProgram>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
pub struct DoubleState {
    value: f64,
}

#[derive(Debug, Clone)]
pub struct Dbl {
    parent: Arc<CnvObject>,
    state: RefCell<DoubleState>,
    initial_properties: DblInit,
}

impl Dbl {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: DblInit) -> Self {
        let value = initial_properties.value.unwrap_or(0f64);
        Self {
            parent,
            state: RefCell::new(DoubleState { value }),
            initial_properties,
        }
    }
}

impl CnvType for Dbl {
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
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("SET") => {
                assert!(arguments.len() == 1);
                self.state
                    .borrow_mut()
                    .set(self, context, arguments[0].to_double())?;
                Ok(None)
            }
            CallableIdentifier::Method("GET") => {
                Ok(Some(CnvValue::Double(self.state.borrow().get()?)))
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONCHANGED") => {
                if let Some(v) = self.initial_properties.on_changed.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONBRUTALCHANGED") => {
                if let Some(v) = self.initial_properties.on_brutal_changed.as_ref() {
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
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
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
        Ok(Self::from_initial_properties(
            parent,
            DblInit {
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

impl DoubleState {
    pub fn add() {
        // ADD
        todo!()
    }

    pub fn arctan() {
        // ARCTAN
        todo!()
    }

    pub fn arctanex() {
        // ARCTANEX
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

    pub fn cosinus() {
        // COSINUS
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

    pub fn get(&self) -> RunnerResult<f64> {
        // GET
        Ok(self.value)
    }

    pub fn inc() {
        // INC
        todo!()
    }

    pub fn length() {
        // LENGTH
        todo!()
    }

    pub fn log() {
        // LOG
        todo!()
    }

    pub fn maxa() {
        // MAXA
        todo!()
    }

    pub fn mina() {
        // MINA
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

    pub fn round() {
        // ROUND
        todo!()
    }

    pub fn set(
        &mut self,
        double: &Dbl,
        context: &mut RunnerContext,
        value: f64,
    ) -> RunnerResult<()> {
        // SET
        let changed_value = self.value != value;
        self.value = value;
        if changed_value {
            double.call_method(
                CallableIdentifier::Event("ONCHANGED"),
                &vec![CnvValue::Double(self.value)],
                context,
            )?;
        }
        double.call_method(
            CallableIdentifier::Event("ONBRUTALCHANGED"),
            &vec![CnvValue::Double(self.value)],
            context,
        )?;
        Ok(())
    }

    pub fn setdefault() {
        // SETDEFAULT
        todo!()
    }

    pub fn sgn() {
        // SGN
        todo!()
    }

    pub fn sinus() {
        // SINUS
        todo!()
    }

    pub fn sqrt() {
        // SQRT
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
}
