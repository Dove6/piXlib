use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_program};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct StringVarProperties {
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
struct StringVarState {
    pub initialized: bool,

    // initialized from properties
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct StringVarEventHandlers {
    pub on_brutal_changed: Option<Arc<ParsedScript>>, // ONBRUTALCHANGED signal
    pub on_changed: Option<Arc<ParsedScript>>,        // ONCHANGED signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_net_changed: Option<Arc<ParsedScript>>,    // ONNETCHANGED signal
    pub on_signal: Option<Arc<ParsedScript>>,         // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct StringVar {
    parent: Arc<CnvObject>,

    state: RefCell<StringVarState>,
    event_handlers: StringVarEventHandlers,

    should_notify_on_net_changed: bool,
    should_be_stored_to_ini: bool,
}

impl StringVar {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: StringVarProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(StringVarState {
                value: props.value.unwrap_or_default(),
                ..Default::default()
            }),
            event_handlers: StringVarEventHandlers {
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

    pub fn get(&self) -> RunnerResult<String> {
        self.state.borrow().get()
    }
}

impl CnvType for StringVar {
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
            CallableIdentifier::Method("ADD") => self.state.borrow_mut().add().map(|_| None),
            CallableIdentifier::Method("CLEAR") => self.state.borrow_mut().clear().map(|_| None),
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file().map(|_| None)
            }
            CallableIdentifier::Method("CUT") => self.state.borrow_mut().cut().map(|_| None),
            CallableIdentifier::Method("FIND") => self.state.borrow_mut().find().map(|_| None),
            CallableIdentifier::Method("GET") => {
                self.state.borrow().get().map(|v| Some(CnvValue::String(v)))
            }
            CallableIdentifier::Method("INSERTAT") => {
                self.state.borrow_mut().insert_at().map(|_| None)
            }
            CallableIdentifier::Method("ISUPPERLETTER") => self
                .state
                .borrow()
                .is_upper_letter()
                .map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Method("LENGTH") => self.state.borrow_mut().length().map(|_| None),
            CallableIdentifier::Method("LOWER") => self.state.borrow_mut().lower().map(|_| None),
            CallableIdentifier::Method("NOT") => self.state.borrow_mut().not().map(|_| None),
            CallableIdentifier::Method("RANDOM") => self.state.borrow_mut().random().map(|_| None),
            CallableIdentifier::Method("REPLACE") => {
                self.state.borrow_mut().replace().map(|_| None)
            }
            CallableIdentifier::Method("REPLACEAT") => {
                self.state.borrow_mut().replace_at().map(|_| None)
            }
            CallableIdentifier::Method("RESETINI") => {
                self.state.borrow_mut().reset_ini().map(|_| None)
            }
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(self, &arguments[0].to_string())
                .map(|_| None),
            CallableIdentifier::Method("SETDEFAULT") => {
                self.state.borrow_mut().set_default().map(|_| None)
            }
            CallableIdentifier::Method("SUB") => self.state.borrow_mut().sub().map(|_| None),
            CallableIdentifier::Method("SWITCH") => self.state.borrow_mut().switch().map(|_| None),
            CallableIdentifier::Method("UPPER") => self.state.borrow_mut().upper().map(|_| None),
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
        Ok(CnvContent::String(StringVar::from_initial_properties(
            parent,
            StringVarProperties {
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

impl StringVarState {
    pub fn add(&mut self) -> RunnerResult<()> {
        // ADD
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

    pub fn cut(&mut self) -> RunnerResult<()> {
        // CUT
        todo!()
    }

    pub fn find(&mut self) -> RunnerResult<()> {
        // FIND
        todo!()
    }

    pub fn get(&self) -> RunnerResult<String> {
        // GET
        Ok(self.value.clone())
    }

    pub fn insert_at(&mut self) -> RunnerResult<()> {
        // INSERTAT
        todo!()
    }

    pub fn is_upper_letter(&self) -> RunnerResult<bool> {
        // ISUPPERLETTER
        todo!()
    }

    pub fn length(&mut self) -> RunnerResult<()> {
        // LENGTH
        todo!()
    }

    pub fn lower(&mut self) -> RunnerResult<()> {
        // LOWER
        todo!()
    }

    pub fn not(&mut self) -> RunnerResult<()> {
        // NOT
        todo!()
    }

    pub fn random(&mut self) -> RunnerResult<()> {
        // RANDOM
        todo!()
    }

    pub fn replace(&mut self) -> RunnerResult<()> {
        // REPLACE
        todo!()
    }

    pub fn replace_at(&mut self) -> RunnerResult<()> {
        // REPLACEAT
        todo!()
    }

    pub fn reset_ini(&mut self) -> RunnerResult<()> {
        // RESETINI
        todo!()
    }

    pub fn set(&mut self, string: &StringVar, value: &str) -> RunnerResult<()> {
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

    pub fn set_default(&mut self) -> RunnerResult<()> {
        // SETDEFAULT
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

    pub fn upper(&mut self) -> RunnerResult<()> {
        // UPPER
        todo!()
    }
}
