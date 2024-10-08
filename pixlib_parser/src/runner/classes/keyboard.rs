use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct KeyboardProperties {
    // KEYBOARD
    pub keyboard: Option<String>, // KEYBOARD

    pub on_char: Option<Arc<ParsedScript>>, // ONCHAR signal
    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_key_down: Option<Arc<ParsedScript>>, // ONKEYDOWN signal
    pub on_key_up: Option<Arc<ParsedScript>>, // ONKEYUP signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct KeyboardState {
    // deduced from methods
    pub is_enabled: bool,
    pub is_auto_repeat_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct KeyboardEventHandlers {
    pub on_char: Option<Arc<ParsedScript>>,     // ONCHAR signal
    pub on_done: Option<Arc<ParsedScript>>,     // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,     // ONINIT signal
    pub on_key_down: Option<Arc<ParsedScript>>, // ONKEYDOWN signal
    pub on_key_up: Option<Arc<ParsedScript>>,   // ONKEYUP signal
    pub on_signal: Option<Arc<ParsedScript>>,   // ONSIGNAL signal
}

impl EventHandler for KeyboardEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONCHAR" => self.on_char.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONKEYDOWN" => self.on_key_down.as_ref(),
            "ONKEYUP" => self.on_key_up.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    parent: Arc<CnvObject>,

    state: RefCell<KeyboardState>,
    event_handlers: KeyboardEventHandlers,

    keyboard: String,
}

impl Keyboard {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: KeyboardProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(KeyboardState {
                is_enabled: true,
                ..Default::default()
            }),
            event_handlers: KeyboardEventHandlers {
                on_char: props.on_char,
                on_done: props.on_done,
                on_init: props.on_init,
                on_key_down: props.on_key_down,
                on_key_up: props.on_key_up,
                on_signal: props.on_signal,
            },
            keyboard: props.keyboard.unwrap_or_default(),
        }
    }
}

impl CnvType for Keyboard {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "KEYBOARD"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("DISABLE") => {
                self.state.borrow_mut().disable().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ENABLE") => {
                self.state.borrow_mut().enable().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETLATESTKEY") => self
                .state
                .borrow_mut()
                .get_latest_key()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETLATESTKEYS") => self
                .state
                .borrow_mut()
                .get_latest_keys()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("ISENABLED") => {
                self.state.borrow().is_enabled().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("ISKEYDOWN") => self
                .state
                .borrow_mut()
                .is_key_down()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETAUTOREPEAT") => self
                .state
                .borrow_mut()
                .set_auto_repeat(arguments[0].to_bool())
                .map(|_| CnvValue::Null),
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
        let keyboard = properties.remove("KEYBOARD").and_then(discard_if_empty);
        let on_char = properties
            .remove("ONCHAR")
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
        let on_key_down = properties
            .remove("ONKEYDOWN")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_key_up = properties
            .remove("ONKEYUP")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::Keyboard(Self::from_initial_properties(
            parent,
            KeyboardProperties {
                keyboard,
                on_char,
                on_done,
                on_init,
                on_key_down,
                on_key_up,
                on_signal,
            },
        )))
    }
}

impl Initable for Keyboard {
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

impl KeyboardState {
    pub fn disable(&mut self) -> anyhow::Result<()> {
        // DISABLE
        self.is_enabled = false;
        Ok(())
    }

    pub fn enable(&mut self) -> anyhow::Result<()> {
        // ENABLE
        self.is_enabled = true;
        Ok(())
    }

    pub fn get_latest_key(&mut self) -> anyhow::Result<()> {
        // GETLATESTKEY
        todo!()
    }

    pub fn get_latest_keys(&mut self) -> anyhow::Result<()> {
        // GETLATESTKEYS
        todo!()
    }

    pub fn is_enabled(&self) -> anyhow::Result<bool> {
        // ISENABLED
        Ok(self.is_enabled)
    }

    pub fn is_key_down(&mut self) -> anyhow::Result<()> {
        // ISKEYDOWN
        todo!()
    }

    pub fn set_auto_repeat(&mut self, enabled: bool) -> anyhow::Result<()> {
        // SETAUTOREPEAT
        self.is_auto_repeat_enabled = enabled;
        Ok(())
    }
}
