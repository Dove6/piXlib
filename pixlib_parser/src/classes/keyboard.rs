use std::{any::Any, cell::RefCell};

use content::EventHandler;
use initable::Initable;
use parsers::{discard_if_empty, parse_event_handler};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

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
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("DISABLE") => {
                self.state.borrow_mut().disable().map(|_| None)
            }
            CallableIdentifier::Method("ENABLE") => self.state.borrow_mut().enable().map(|_| None),
            CallableIdentifier::Method("GETLATESTKEY") => {
                self.state.borrow_mut().get_latest_key().map(|_| None)
            }
            CallableIdentifier::Method("GETLATESTKEYS") => {
                self.state.borrow_mut().get_latest_keys().map(|_| None)
            }
            CallableIdentifier::Method("ISENABLED") => {
                self.state.borrow_mut().is_enabled().map(|_| None)
            }
            CallableIdentifier::Method("ISKEYDOWN") => {
                self.state.borrow_mut().is_key_down().map(|_| None)
            }
            CallableIdentifier::Method("SETAUTOREPEAT") => {
                self.state.borrow_mut().set_auto_repeat().map(|_| None)
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

impl KeyboardState {
    pub fn disable(&mut self) -> RunnerResult<()> {
        // DISABLE
        todo!()
    }

    pub fn enable(&mut self) -> RunnerResult<()> {
        // ENABLE
        todo!()
    }

    pub fn get_latest_key(&mut self) -> RunnerResult<()> {
        // GETLATESTKEY
        todo!()
    }

    pub fn get_latest_keys(&mut self) -> RunnerResult<()> {
        // GETLATESTKEYS
        todo!()
    }

    pub fn is_enabled(&mut self) -> RunnerResult<()> {
        // ISENABLED
        todo!()
    }

    pub fn is_key_down(&mut self) -> RunnerResult<()> {
        // ISKEYDOWN
        todo!()
    }

    pub fn set_auto_repeat(&mut self) -> RunnerResult<()> {
        // SETAUTOREPEAT
        todo!()
    }
}
