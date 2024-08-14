use std::{any::Any, cell::RefCell};

use content::EventHandler;
use initable::Initable;
use parsers::{discard_if_empty, parse_event_handler, parse_i32, Rect};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

use super::*;

#[derive(Debug, Clone)]
pub struct MouseProperties {
    // MOUSE
    pub mouse: Option<String>, // MOUSE
    pub raw: Option<i32>,      // RAW

    pub on_click: Option<Arc<ParsedScript>>, // ONCLICK signal
    pub on_dbl_click: Option<Arc<ParsedScript>>, // ONDBLCLICK signal
    pub on_done: Option<Arc<ParsedScript>>,  // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,  // ONINIT signal
    pub on_move: Option<Arc<ParsedScript>>,  // ONMOVE signal
    pub on_release: Option<Arc<ParsedScript>>, // ONRELEASE signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct MouseState {
    // deduced from methods
    is_enabled: bool,
    are_events_enabled: bool,
    is_visible: bool,
    clip_rect: Option<Rect>,
}

#[derive(Debug, Clone)]
pub struct MouseEventHandlers {
    pub on_click: Option<Arc<ParsedScript>>,     // ONCLICK signal
    pub on_dbl_click: Option<Arc<ParsedScript>>, // ONDBLCLICK signal
    pub on_done: Option<Arc<ParsedScript>>,      // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,      // ONINIT signal
    pub on_move: Option<Arc<ParsedScript>>,      // ONMOVE signal
    pub on_release: Option<Arc<ParsedScript>>,   // ONRELEASE signal
    pub on_signal: Option<Arc<ParsedScript>>,    // ONSIGNAL signal
}

impl EventHandler for MouseEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONCLICK" => self.on_click.as_ref(),
            "ONDBLCLICK" => self.on_dbl_click.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONMOVE" => self.on_move.as_ref(),
            "ONRELEASE" => self.on_release.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mouse {
    parent: Arc<CnvObject>,

    state: RefCell<MouseState>,
    event_handlers: MouseEventHandlers,

    mouse: String,
    raw: i32,
}

impl Mouse {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: MouseProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(MouseState {
                is_enabled: true,
                are_events_enabled: true,
                is_visible: true,
                ..Default::default()
            }),
            event_handlers: MouseEventHandlers {
                on_click: props.on_click,
                on_dbl_click: props.on_dbl_click,
                on_done: props.on_done,
                on_init: props.on_init,
                on_move: props.on_move,
                on_release: props.on_release,
                on_signal: props.on_signal,
            },
            mouse: props.mouse.unwrap_or_default(),
            raw: props.raw.unwrap_or_default(),
        }
    }
}

impl CnvType for Mouse {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "MOUSE"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("CLICK") => self.state.borrow_mut().click().map(|_| None),
            CallableIdentifier::Method("DISABLE") => {
                self.state.borrow_mut().disable().map(|_| None)
            }
            CallableIdentifier::Method("DISABLESIGNAL") => {
                self.state.borrow_mut().disable_signal().map(|_| None)
            }
            CallableIdentifier::Method("ENABLE") => self.state.borrow_mut().enable().map(|_| None),
            CallableIdentifier::Method("ENABLESIGNAL") => {
                self.state.borrow_mut().enable_signal().map(|_| None)
            }
            CallableIdentifier::Method("GETLASTCLICKPOSX") => self
                .state
                .borrow()
                .get_last_click_pos_x()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETLASTCLICKPOSY") => self
                .state
                .borrow()
                .get_last_click_pos_y()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPOSX") => self
                .state
                .borrow()
                .get_pos_x()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPOSY") => self
                .state
                .borrow()
                .get_pos_y()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("HIDE") => self.state.borrow_mut().hide().map(|_| None),
            CallableIdentifier::Method("ISLBUTTONDOWN") => self
                .state
                .borrow()
                .is_l_button_down()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("ISRBUTTONDOWN") => self
                .state
                .borrow()
                .is_r_button_down()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("LOCKACTIVECURSOR") => {
                self.state.borrow_mut().lock_active_cursor().map(|_| None)
            }
            CallableIdentifier::Method("MOUSERELEASE") => {
                self.state.borrow_mut().mouse_release().map(|_| None)
            }
            CallableIdentifier::Method("MOVE") => self.state.borrow_mut().move_by().map(|_| None),
            CallableIdentifier::Method("SET") => self.state.borrow_mut().set().map(|_| None),
            CallableIdentifier::Method("SETACTIVERECT") => {
                self.state.borrow_mut().set_active_rect().map(|_| None)
            }
            CallableIdentifier::Method("SETCLIPRECT") => {
                self.state.borrow_mut().set_clip_rect().map(|_| None)
            }
            CallableIdentifier::Method("SETPOSITION") => {
                self.state.borrow_mut().set_position().map(|_| None)
            }
            CallableIdentifier::Method("SHOW") => self.state.borrow_mut().show().map(|_| None),
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
        let mouse = properties.remove("MOUSE").and_then(discard_if_empty);
        let raw = properties
            .remove("RAW")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let on_click = properties
            .remove("ONCLICK")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_dbl_click = properties
            .remove("ONDBLCLICK")
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
        let on_move = properties
            .remove("ONMOVE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_release = properties
            .remove("ONRELEASE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::Mouse(Self::from_initial_properties(
            parent,
            MouseProperties {
                mouse,
                raw,
                on_click,
                on_dbl_click,
                on_done,
                on_init,
                on_move,
                on_release,
                on_signal,
            },
        )))
    }
}

impl Initable for Mouse {
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

impl MouseState {
    pub fn click(&mut self) -> RunnerResult<()> {
        // CLICK
        todo!()
    }

    pub fn disable(&mut self) -> RunnerResult<()> {
        // DISABLE
        todo!()
    }

    pub fn disable_signal(&mut self) -> RunnerResult<()> {
        // DISABLESIGNAL
        todo!()
    }

    pub fn enable(&mut self) -> RunnerResult<()> {
        // ENABLE
        todo!()
    }

    pub fn enable_signal(&mut self) -> RunnerResult<()> {
        // ENABLESIGNAL
        todo!()
    }

    pub fn get_last_click_pos_x(&self) -> RunnerResult<isize> {
        // GETLASTCLICKPOSX
        todo!()
    }

    pub fn get_last_click_pos_y(&self) -> RunnerResult<isize> {
        // GETLASTCLICKPOSY
        todo!()
    }

    pub fn get_pos_x(&self) -> RunnerResult<isize> {
        // GETPOSX
        todo!()
    }

    pub fn get_pos_y(&self) -> RunnerResult<isize> {
        // GETPOSY
        todo!()
    }

    pub fn hide(&mut self) -> RunnerResult<()> {
        // HIDE
        todo!()
    }

    pub fn is_l_button_down(&self) -> RunnerResult<bool> {
        // ISLBUTTONDOWN
        todo!()
    }

    pub fn is_r_button_down(&self) -> RunnerResult<bool> {
        // ISRBUTTONDOWN
        todo!()
    }

    pub fn lock_active_cursor(&mut self) -> RunnerResult<()> {
        // LOCKACTIVECURSOR
        todo!()
    }

    pub fn mouse_release(&mut self) -> RunnerResult<()> {
        // MOUSERELEASE
        todo!()
    }

    pub fn move_by(&mut self) -> RunnerResult<()> {
        // MOVE
        todo!()
    }

    pub fn set(&mut self) -> RunnerResult<()> {
        // SET
        todo!()
    }

    pub fn set_active_rect(&mut self) -> RunnerResult<()> {
        // SETACTIVERECT
        todo!()
    }

    pub fn set_clip_rect(&mut self) -> RunnerResult<()> {
        // SETCLIPRECT
        todo!()
    }

    pub fn set_position(&mut self) -> RunnerResult<()> {
        // SETPOSITION
        todo!()
    }

    pub fn show(&mut self) -> RunnerResult<()> {
        // SHOW
        todo!()
    }
}
