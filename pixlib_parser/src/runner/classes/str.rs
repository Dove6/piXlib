use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_bool, parse_event_handler};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
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
    // initialized from properties
    pub default_value: String,
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

impl EventHandler for StringVarEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONBRUTALCHANGED" => self.on_brutal_changed.as_ref(),
            "ONCHANGED" => self.on_changed.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONNETCHANGED" => self.on_net_changed.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
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
        let value = props.value.unwrap_or_default();
        Self {
            parent,
            state: RefCell::new(StringVarState {
                default_value: props.default.unwrap_or(value.clone()),
                value,
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
        self.state.borrow().get(None, None)
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // eprintln!(
        //     "Calling method {:?} with arguments [{}]",
        //     name,
        //     arguments.iter().join(", ")
        // );
        match name {
            CallableIdentifier::Method("ADD") => self
                .state
                .borrow_mut()
                .add(context, &arguments[0].to_str())
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("CLEAR") => {
                self.state.borrow_mut().clear(context).map(|_| None)
            }
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file(context).map(|_| None)
            }
            CallableIdentifier::Method("CUT") => self
                .state
                .borrow_mut()
                .cut(
                    context,
                    arguments[0].to_int() as usize,
                    arguments[1].to_int() as usize,
                )
                .map(|_| None),
            CallableIdentifier::Method("FIND") => self
                .state
                .borrow()
                .find(
                    &arguments[0].to_str(),
                    arguments.get(1).map(|v| v.to_int() as usize),
                )
                .map(|v| v.map(|u| u as i32).unwrap_or(-1))
                .map(|v| Some(CnvValue::Integer(v))),
            CallableIdentifier::Method("GET") => self
                .state
                .borrow()
                .get(
                    arguments.first().map(|v| v.to_int() as usize),
                    arguments.get(1).map(|v| v.to_int() as usize),
                )
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("INSERTAT") => self
                .state
                .borrow_mut()
                .insert_at(
                    context,
                    arguments[0].to_int() as usize,
                    &arguments[1].to_str(),
                    arguments.get(2).map(|v| v.to_int() as usize).unwrap_or(1),
                )
                .map(|_| None),
            CallableIdentifier::Method("ISUPPERLETTER") => self
                .state
                .borrow()
                .is_upper_letter(arguments[0].to_int() as usize)
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("LENGTH") => self
                .state
                .borrow_mut()
                .length()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("LOWER") => {
                self.state.borrow_mut().lower(context).map(|_| None)
            }
            CallableIdentifier::Method("NOT") => self.state.borrow_mut().not(context).map(|_| None),
            CallableIdentifier::Method("RANDOM") => {
                self.state.borrow_mut().random(context).map(|_| None)
            }
            CallableIdentifier::Method("REPLACE") => self
                .state
                .borrow_mut()
                .replace(context, &arguments[0].to_str(), &arguments[1].to_str())
                .map(|_| None),
            CallableIdentifier::Method("REPLACEAT") => self
                .state
                .borrow_mut()
                .replace_at(
                    context,
                    arguments[0].to_int() as usize,
                    &arguments[1].to_str(),
                )
                .map(|_| None),
            CallableIdentifier::Method("RESETINI") => {
                self.state.borrow_mut().reset_ini(context).map(|_| None)
            }
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(context, &arguments[0].to_str())
                .map(|_| None),
            CallableIdentifier::Method("SETDEFAULT") => self
                .state
                .borrow_mut()
                .set_default(context, &arguments[0].to_str())
                .map(|_| None),
            CallableIdentifier::Method("SUB") => self
                .state
                .borrow_mut()
                .sub(
                    context,
                    arguments[0].to_int() as usize,
                    arguments[1].to_int() as usize,
                )
                .map(|_| None),
            CallableIdentifier::Method("SWITCH") => self
                .state
                .borrow_mut()
                .switch(context, &arguments[0].to_str(), &arguments[1].to_str())
                .map(|_| None),
            CallableIdentifier::Method("UPPER") => {
                self.state.borrow_mut().upper(context).map(|_| None)
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new_content(
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
            .map(parse_event_handler)
            .transpose()?;
        let on_changed = properties
            .remove("ONCHANGED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
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
        let on_net_changed = properties
            .remove("ONNETCHANGED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
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

impl Initable for StringVar {
    fn initialize(&mut self, context: RunnerContext) -> RunnerResult<()> {
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

impl StringVarState {
    pub fn add(&mut self, context: RunnerContext, suffix: &str) -> RunnerResult<String> {
        // ADD
        self.change_value(context, self.value.clone() + suffix);
        Ok(self.value.clone())
    }

    pub fn clear(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // CLEAR
        self.change_value(context, "".to_owned());
        Ok(())
    }

    pub fn copy_file(&mut self, _context: RunnerContext) -> RunnerResult<bool> {
        // COPYFILE
        todo!()
    }

    pub fn cut(&mut self, context: RunnerContext, index: usize, length: usize) -> RunnerResult<()> {
        // CUT
        let value = if length > 0 {
            self.value[index..(index + length)].to_owned()
        } else {
            self.value[index..].to_owned()
        };
        self.value = value; // doesn't emit onchanged
        self.change_value(context, self.value.clone());
        Ok(())
    }

    pub fn find(&self, needle: &str, start_index: Option<usize>) -> RunnerResult<Option<usize>> {
        // FIND
        Ok(self
            .value
            .match_indices(needle)
            .find(|m| {
                if let Some(start_index) = start_index {
                    m.0 >= start_index
                } else {
                    true
                }
            })
            .map(|m| m.0))
    }

    pub fn get(&self, index: Option<usize>, length: Option<usize>) -> RunnerResult<String> {
        // GET
        let index = index.unwrap_or_default();
        let length = length.unwrap_or(self.value.len() - index);
        Ok(self.value[index..(index + length)].to_owned())
    }

    pub fn insert_at(
        &mut self,
        context: RunnerContext,
        index: usize,
        value: &str,
        times: usize,
    ) -> RunnerResult<()> {
        // INSERTAT
        if times == 0 || value.is_empty() {
            return Ok(());
        }
        for _ in 0..times {
            self.value.insert_str(index, value); // doesn't emit onchanged
        }
        self.change_value(context, self.value.clone());
        todo!()
    }

    pub fn is_upper_letter(&self, index: usize) -> RunnerResult<bool> {
        // ISUPPERLETTER
        Ok(self
            .value
            .as_bytes()
            .get(index)
            .copied()
            .map(|b| b.is_ascii_uppercase())
            .unwrap_or_default())
    }

    pub fn length(&self) -> RunnerResult<usize> {
        // LENGTH
        Ok(self.value.len())
    }

    pub fn lower(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // LOWER
        self.change_value(context, self.value.to_ascii_lowercase());
        Ok(())
    }

    pub fn not(&mut self, context: RunnerContext) -> RunnerResult<String> {
        // NOT
        self.value = String::from_utf8(self.value.bytes().rev().collect()).unwrap(); // doesn't emit onchanged
        self.change_value(context, self.value.clone());
        Ok(self.value.clone())
    }

    pub fn random(&mut self, _context: RunnerContext) -> RunnerResult<i32> {
        // RANDOM
        todo!()
    }

    pub fn replace(
        &mut self,
        context: RunnerContext,
        search: &str,
        replace: &str,
    ) -> RunnerResult<()> {
        // REPLACE
        std::mem::drop(self.value.replace(search, replace)); // doesn't emit onchanged
        self.change_value(context, self.value.clone()); // but emits onbrutalchanged even when not changed
        Ok(())
    }

    pub fn replace_at(
        &mut self,
        context: RunnerContext,
        index: usize,
        replace: &str,
    ) -> RunnerResult<()> {
        // REPLACEAT
        std::mem::drop(self.value.replace(&self.value[index..].to_owned(), replace)); // doesn't emit onchanged
        self.change_value(context, self.value.clone()); // but emits onbrutalchanged even when not changed
        Ok(())
    }

    pub fn reset_ini(&mut self, _context: RunnerContext) -> RunnerResult<()> {
        // RESETINI
        todo!()
    }

    pub fn set(&mut self, context: RunnerContext, value: &str) -> RunnerResult<()> {
        // SET
        self.change_value(context, value.to_owned());
        Ok(())
    }

    pub fn set_default(
        &mut self,
        _context: RunnerContext,
        default_value: &str,
    ) -> RunnerResult<()> {
        // SETDEFAULT
        self.default_value = default_value.to_owned();
        Ok(())
    }

    pub fn sub(&mut self, context: RunnerContext, index: usize, length: usize) -> RunnerResult<()> {
        // SUB
        self.value.drain(index..(index + length)); // doesn't emit onchanged
        self.change_value(context, self.value.clone()); // but emits onbrutalchanged even when not changed
        Ok(())
    }

    pub fn switch(
        &mut self,
        context: RunnerContext,
        first: &str,
        second: &str,
    ) -> RunnerResult<()> {
        // SWITCH
        self.change_value(
            context,
            if self.value == first {
                second.to_owned()
            } else {
                first.to_owned()
            },
        );
        Ok(())
    }

    pub fn upper(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // UPPER
        self.change_value(context, self.value.to_ascii_uppercase());
        Ok(())
    }

    // custom

    fn change_value(&mut self, context: RunnerContext, value: String) {
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
                    arguments: vec![CnvValue::String(self.value.clone())],
                });
                if changed {
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifier::Event("ONCHANGED").to_owned(),
                        arguments: vec![CnvValue::String(self.value.clone())],
                    });
                }
            });
    }
}
