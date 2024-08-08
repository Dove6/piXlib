use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_program};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct StrInit {
    // STRING
    pub default: Option<String>,  // DEFAULT
    pub net_notify: Option<bool>, // NETNOTIFY
    pub to_ini: Option<bool>,     // TOINI
    pub value: Option<String>,    // VALUE

    pub on_brutal_changed: Option<Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct StringState {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Str {
    parent: Arc<CnvObject>,
    state: RefCell<StringState>,
    initial_properties: StrInit,
}

impl Str {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: StrInit) -> Self {
        let value = initial_properties.value.clone().unwrap_or(String::new());
        Self {
            parent,
            state: RefCell::new(StringState { value }),
            initial_properties,
        }
    }
}

impl CnvType for Str {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "STRING"
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
            CallableIdentifier::Method("SET") => {
                assert!(arguments.len() == 1);
                self.state
                    .borrow_mut()
                    .set(self, &arguments[0].to_string())?;
                Ok(None)
            }
            CallableIdentifier::Method("GET") => {
                Ok(Some(CnvValue::String(self.state.borrow().get()?)))
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONCHANGED") => {
                if let Some(v) = self.initial_properties.on_changed.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONBRUTALCHANGED") => {
                if let Some(v) = self.initial_properties.on_brutal_changed.as_ref() {
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
    ) -> Result<Self, TypeParsingError> {
        let default = properties.remove("DEFAULT").and_then(discard_if_empty);
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
        let value = properties.remove("VALUE");
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
            StrInit {
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

impl StringState {
    pub fn add() {
        // ADD
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

    pub fn cut() {
        // CUT
        todo!()
    }

    pub fn find() {
        // FIND
        todo!()
    }

    pub fn get(&self) -> RunnerResult<String> {
        // GET
        Ok(self.value.clone())
    }

    pub fn insertat() {
        // INSERTAT
        todo!()
    }

    pub fn isupperletter() {
        // ISUPPERLETTER
        todo!()
    }

    pub fn length() {
        // LENGTH
        todo!()
    }

    pub fn lower() {
        // LOWER
        todo!()
    }

    pub fn not() {
        // NOT
        todo!()
    }

    pub fn random() {
        // RANDOM
        todo!()
    }

    pub fn replace() {
        // REPLACE
        todo!()
    }

    pub fn replaceat() {
        // REPLACEAT
        todo!()
    }

    pub fn resetini() {
        // RESETINI
        todo!()
    }

    pub fn set(&mut self, string: &Str, value: &str) -> RunnerResult<()> {
        // SET
        let changed_value = self.value != value;
        self.value = value.to_owned();
        let context = RunnerContext {
            runner: Arc::clone(&string.parent.parent.runner),
            self_object: string.parent.name.clone(),
            current_object: string.parent.name.clone(),
        };
        if changed_value {
            string.call_method(
                CallableIdentifier::Event("ONCHANGED"),
                &vec![CnvValue::String(self.value.clone())],
                context.clone(),
            )?;
        }
        string.call_method(
            CallableIdentifier::Event("ONBRUTALCHANGED"),
            &vec![CnvValue::String(self.value.clone())],
            context,
        )?;
        Ok(())
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

    pub fn upper() {
        // UPPER
        todo!()
    }
}
