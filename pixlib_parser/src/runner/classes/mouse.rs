use std::{
    any::Any,
    collections::VecDeque,
    sync::RwLock,
    time::{SystemTime, UNIX_EPOCH},
};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler, parse_i32, Rect};

use crate::{
    common::DroppableRefMut,
    parser::ast::ParsedScript,
    runner::{InternalEvent, MouseEvent},
};

use super::super::common::*;
use super::super::*;
use super::*;

const DOUBLE_CLICK_MAX_INTERVAL_SECONDS: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct MouseProperties {
    // MOUSE
    pub mouse: Option<String>, // MOUSE
    pub raw: Option<i32>,      // RAW

    pub on_click: HashMap<String, Arc<ParsedScript>>, // ONCLICK signal
    pub on_dbl_click: HashMap<String, Arc<ParsedScript>>, // ONDBLCLICK signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_move: Option<Arc<ParsedScript>>,           // ONMOVE signal
    pub on_release: HashMap<String, Arc<ParsedScript>>, // ONRELEASE signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct MouseState {
    // deduced from methods
    is_enabled: bool,
    are_events_enabled: bool, // TODO: use this
    is_visible: bool,
    clip_rect: Option<Rect>,

    position: (isize, isize),
    last_click_position: (isize, isize),
    last_click_time_seconds: f64,
    is_left_button_down: bool,
    is_right_button_down: bool,
    is_locked: bool,

    events_out: VecDeque<InternalMouseEvent>,
}

#[derive(Debug, Clone)]
pub struct MouseEventHandlers {
    pub on_click: HashMap<String, Arc<ParsedScript>>, // ONCLICK signal
    pub on_dbl_click: HashMap<String, Arc<ParsedScript>>, // ONDBLCLICK signal
    pub on_done: Option<Arc<ParsedScript>>,           // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,           // ONINIT signal
    pub on_move: Option<Arc<ParsedScript>>,           // ONMOVE signal
    pub on_release: HashMap<String, Arc<ParsedScript>>, // ONRELEASE signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
}

impl EventHandler for MouseEventHandlers {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONCLICK" => argument
                .and_then(|a| self.on_click.get(a))
                .or(self.on_click.get("")),
            "ONDBLCLICK" => argument
                .and_then(|a| self.on_dbl_click.get(a))
                .or(self.on_dbl_click.get("")),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONMOVE" => self.on_move.as_ref(),
            "ONRELEASE" => argument
                .and_then(|a| self.on_release.get(a))
                .or(self.on_release.get("")),
            "ONSIGNAL" => argument
                .and_then(|a| self.on_signal.get(a))
                .or(self.on_signal.get("")),
            _ => None,
        }
    }
}

lazy_static! {
    static ref GLOBAL_MOUSE_STATE: Arc<RwLock<MouseState>> = Arc::new(RwLock::new(MouseState {
        is_enabled: true,
        are_events_enabled: true,
        is_visible: true,
        ..Default::default()
    }));
}

#[derive(Debug, Clone)]
pub struct Mouse {
    parent: Arc<CnvObject>,

    state: Arc<RwLock<MouseState>>,
    event_handlers: MouseEventHandlers,

    mouse: String,
    raw: i32,
}

impl Mouse {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: MouseProperties) -> Self {
        Self {
            parent,
            state: Arc::clone(&GLOBAL_MOUSE_STATE),
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

    pub fn handle_incoming_event(event: MouseEvent) -> RunnerResult<()> {
        let mut mouse_state = GLOBAL_MOUSE_STATE.write().unwrap();
        match event {
            MouseEvent::MovedTo { x, y } => mouse_state.set_position(x, y),
            MouseEvent::LeftButtonPressed => mouse_state.set_left_button_down(true),
            MouseEvent::LeftButtonReleased => mouse_state.set_left_button_down(false),
            MouseEvent::RightButtonPressed => mouse_state.set_right_button_down(true),
            MouseEvent::RightButtonReleased => mouse_state.set_right_button_down(false),
        }
    }

    pub fn handle_outgoing_events(
        mut handler: impl FnMut(InternalMouseEvent) -> RunnerResult<()>,
    ) -> RunnerResult<()> {
        let mut mouse_state = GLOBAL_MOUSE_STATE.write().unwrap();
        for event in mouse_state.events_out.drain(..) {
            handler(event)?;
        }
        Ok(())
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
            CallableIdentifier::Method("CLICK") => self
                .state
                .write()
                .unwrap()
                .click_left_button()
                .map(|_| None),
            CallableIdentifier::Method("DISABLE") => {
                self.state.write().unwrap().disable().map(|_| None)
            }
            CallableIdentifier::Method("DISABLESIGNAL") => self
                .state
                .write()
                .unwrap()
                .disable_event_handling()
                .map(|_| None),
            CallableIdentifier::Method("ENABLE") => {
                self.state.write().unwrap().enable().map(|_| None)
            }
            CallableIdentifier::Method("ENABLESIGNAL") => self
                .state
                .write()
                .unwrap()
                .enable_event_handling()
                .map(|_| None),
            CallableIdentifier::Method("GETLASTCLICKPOSX") => self
                .state
                .read()
                .unwrap()
                .get_last_click_position_x()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETLASTCLICKPOSY") => self
                .state
                .read()
                .unwrap()
                .get_last_click_position_y()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPOSX") => self
                .state
                .read()
                .unwrap()
                .get_position_x()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPOSY") => self
                .state
                .read()
                .unwrap()
                .get_position_y()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("HIDE") => self.state.write().unwrap().hide().map(|_| None),
            CallableIdentifier::Method("ISLBUTTONDOWN") => self
                .state
                .read()
                .unwrap()
                .is_left_button_down()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("ISRBUTTONDOWN") => self
                .state
                .read()
                .unwrap()
                .is_right_button_down()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("LOCKACTIVECURSOR") => {
                self.state.write().unwrap().lock_cursor().map(|_| None)
            }
            CallableIdentifier::Method("MOUSERELEASE") => self
                .state
                .write()
                .unwrap()
                .release_left_button()
                .map(|_| None),
            CallableIdentifier::Method("MOVE") => self
                .state
                .write()
                .unwrap()
                .move_by(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| None),
            CallableIdentifier::Method("SET") => self.state.write().unwrap().set().map(|_| None),
            CallableIdentifier::Method("SETACTIVERECT") => {
                self.state.write().unwrap().set_active_rect().map(|_| None)
            }
            CallableIdentifier::Method("SETCLIPRECT") => {
                self.state.write().unwrap().set_clip_rect().map(|_| None)
            }
            CallableIdentifier::Method("SETPOSITION") => self
                .state
                .write()
                .unwrap()
                .set_position(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| None),
            CallableIdentifier::Method("SHOW") => self.state.write().unwrap().show().map(|_| None),
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
        let mouse = properties.remove("MOUSE").and_then(discard_if_empty);
        let raw = properties
            .remove("RAW")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let mut on_click = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONCLICK" {
                on_click.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONCLICK^") {
                on_click.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_dbl_click = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONDBLCLICK" {
                on_dbl_click.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONDBLCLICK^") {
                on_dbl_click.insert(String::from(argument), parse_event_handler(v.to_owned())?);
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
        let on_move = properties
            .remove("ONMOVE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_release = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONRELEASE" {
                on_release.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONRELEASE^") {
                on_release.insert(String::from(argument), parse_event_handler(v.to_owned())?);
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
    pub fn click_left_button(&mut self) -> RunnerResult<()> {
        // CLICK
        self.set_left_button_down(true)
    }

    pub fn disable(&mut self) -> RunnerResult<()> {
        // DISABLE
        self.is_enabled = false;
        Ok(())
    }

    pub fn disable_event_handling(&mut self) -> RunnerResult<()> {
        // DISABLESIGNAL
        self.are_events_enabled = false;
        Ok(())
    }

    pub fn enable(&mut self) -> RunnerResult<()> {
        // ENABLE
        self.is_enabled = true;
        Ok(())
    }

    pub fn enable_event_handling(&mut self) -> RunnerResult<()> {
        // ENABLESIGNAL
        self.are_events_enabled = true;
        Ok(())
    }

    pub fn get_last_click_position_x(&self) -> RunnerResult<isize> {
        // GETLASTCLICKPOSX
        Ok(self.last_click_position.0)
    }

    pub fn get_last_click_position_y(&self) -> RunnerResult<isize> {
        // GETLASTCLICKPOSY
        Ok(self.last_click_position.1)
    }

    pub fn get_position_x(&self) -> RunnerResult<isize> {
        // GETPOSX
        Ok(self.position.0)
    }

    pub fn get_position_y(&self) -> RunnerResult<isize> {
        // GETPOSY
        Ok(self.position.1)
    }

    pub fn hide(&mut self) -> RunnerResult<()> {
        // HIDE
        self.is_visible = false;
        Ok(())
    }

    pub fn is_left_button_down(&self) -> RunnerResult<bool> {
        // ISLBUTTONDOWN
        Ok(self.is_left_button_down)
    }

    pub fn is_right_button_down(&self) -> RunnerResult<bool> {
        // ISRBUTTONDOWN
        Ok(self.is_right_button_down)
    }

    pub fn lock_cursor(&mut self) -> RunnerResult<()> {
        // LOCKACTIVECURSOR
        self.is_locked = true;
        self.events_out.push_back(InternalMouseEvent::CursorLocked);
        Ok(())
    }

    pub fn release_left_button(&mut self) -> RunnerResult<()> {
        // MOUSERELEASE
        self.set_left_button_down(false)
    }

    pub fn move_by(&mut self, x: isize, y: isize) -> RunnerResult<()> {
        // MOVE
        self.position = (self.position.0 + x, self.position.1 + y);
        self.events_out
            .push_back(InternalMouseEvent::MovedBy { x, y });
        Ok(())
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

    pub fn set_position(&mut self, x: isize, y: isize) -> RunnerResult<()> {
        // SETPOSITION
        let position_diff = (x - self.position.0, y - self.position.1);
        self.position = (x, y);
        if position_diff.0 != 0 && position_diff.1 != 0 {
            self.events_out.push_back(InternalMouseEvent::MovedBy {
                x: position_diff.0,
                y: position_diff.1,
            });
        }
        Ok(())
    }

    pub fn show(&mut self) -> RunnerResult<()> {
        // SHOW
        self.is_visible = true;
        Ok(())
    }

    // custom

    pub fn set_left_button_down(&mut self, is_down: bool) -> RunnerResult<()> {
        if is_down != self.is_left_button_down {
            if is_down {
                self.events_out
                    .push_back(InternalMouseEvent::LeftButtonPressed {
                        x: self.position.0,
                        y: self.position.1,
                    });
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                if now - self.last_click_time_seconds <= DOUBLE_CLICK_MAX_INTERVAL_SECONDS {
                    self.events_out
                        .push_back(InternalMouseEvent::LeftButtonDoubleClicked {
                            x: self.position.0,
                            y: self.position.1,
                        });
                }
                self.last_click_position = self.position;
                self.last_click_time_seconds = now;
            } else {
                self.events_out
                    .push_back(InternalMouseEvent::LeftButtonReleased {
                        x: self.position.0,
                        y: self.position.1,
                    });
            }
        }
        self.is_left_button_down = is_down;
        Ok(())
    }

    pub fn set_right_button_down(&mut self, is_down: bool) -> RunnerResult<()> {
        if is_down != self.is_right_button_down {
            if is_down {
                self.events_out
                    .push_back(InternalMouseEvent::RightButtonPressed {
                        x: self.position.0,
                        y: self.position.1,
                    });
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                if now - self.last_click_time_seconds <= DOUBLE_CLICK_MAX_INTERVAL_SECONDS {
                    self.events_out
                        .push_back(InternalMouseEvent::RightButtonDoubleClicked {
                            x: self.position.0,
                            y: self.position.1,
                        });
                }
                self.last_click_position = self.position;
                self.last_click_time_seconds = now;
            } else {
                self.events_out
                    .push_back(InternalMouseEvent::RightButtonReleased {
                        x: self.position.0,
                        y: self.position.1,
                    });
            }
        }
        self.is_right_button_down = is_down;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum InternalMouseEvent {
    LeftButtonPressed { x: isize, y: isize },
    LeftButtonReleased { x: isize, y: isize },
    RightButtonPressed { x: isize, y: isize },
    RightButtonReleased { x: isize, y: isize },
    LeftButtonDoubleClicked { x: isize, y: isize },
    RightButtonDoubleClicked { x: isize, y: isize },
    MovedBy { x: isize, y: isize },
    CursorLocked,
    CursorReleased,
}
