use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{
    discard_if_empty, parse_bool, parse_event_handler, parse_i32, parse_rect, Rect,
};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ButtonProperties {
    // BUTTON
    pub accent: Option<bool>,            // ACCENT
    pub drag: Option<bool>,              // DRAG
    pub draggable: Option<bool>,         // DRAGGABLE
    pub enable: Option<bool>,            // ENABLE
    pub gfx_on_click: Option<ImageName>, // GFXONCLICK
    pub gfx_on_move: Option<ImageName>,  // GFXONMOVE
    pub gfx_standard: Option<ImageName>, // GFXSTANDARD
    pub priority: Option<i32>,           // PRIORITY
    pub rect: Option<Rect>,              // RECT
    pub snd_on_click: Option<SoundName>, // SNDONCLICK
    pub snd_on_move: Option<SoundName>,  // SNDONMOVE
    pub snd_standard: Option<SoundName>, // SNDSTANDARD

    pub on_action: Option<Arc<ParsedScript>>, // ONACTION signal
    pub on_clicked: Option<Arc<ParsedScript>>, // ONCLICKED signal
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_dragging: Option<Arc<ParsedScript>>, // ONDRAGGING signal
    pub on_end_dragging: Option<Arc<ParsedScript>>, // ONENDDRAGGING signal
    pub on_focus_off: Option<Arc<ParsedScript>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<ParsedScript>>, // ONFOCUSON signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_paused: Option<Arc<ParsedScript>>, // ONPAUSED signal
    pub on_released: Option<Arc<ParsedScript>>, // ONRELEASED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_start_dragging: Option<Arc<ParsedScript>>, // ONSTARTDRAGGING signal
}

#[derive(Debug, Clone, Default)]
pub struct ButtonState {
    // initialized from properties
    pub is_enabled: bool,
    pub is_accented: bool,
    pub is_draggable: bool,
    pub graphics_normal: Option<String>,
    pub graphics_on_hover: Option<String>,
    pub graphics_on_click: Option<String>,
    pub priority: isize,
    pub rect: Option<Rect>,

    // deduced from methods
    pub is_visible: bool,
}

#[derive(Debug, Clone)]
pub struct ButtonEventHandlers {
    pub on_action: Option<Arc<ParsedScript>>,  // ONACTION signal
    pub on_clicked: Option<Arc<ParsedScript>>, // ONCLICKED signal
    pub on_done: Option<Arc<ParsedScript>>,    // ONDONE signal
    pub on_dragging: Option<Arc<ParsedScript>>, // ONDRAGGING signal
    pub on_end_dragging: Option<Arc<ParsedScript>>, // ONENDDRAGGING signal
    pub on_focus_off: Option<Arc<ParsedScript>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<ParsedScript>>, // ONFOCUSON signal
    pub on_init: Option<Arc<ParsedScript>>,    // ONINIT signal
    pub on_paused: Option<Arc<ParsedScript>>,  // ONPAUSED signal
    pub on_released: Option<Arc<ParsedScript>>, // ONRELEASED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_start_dragging: Option<Arc<ParsedScript>>, // ONSTARTDRAGGING signal
}

impl EventHandler for ButtonEventHandlers {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONACTION" => self.on_action.as_ref(),
            "ONCLICKED" => self.on_clicked.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONDRAGGING" => self.on_dragging.as_ref(),
            "ONENDDRAGGING" => self.on_end_dragging.as_ref(),
            "ONFOCUSOFF" => self.on_focus_off.as_ref(),
            "ONFOCUSON" => self.on_focus_on.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONPAUSED" => self.on_paused.as_ref(),
            "ONRELEASED" => self.on_released.as_ref(),
            "ONSIGNAL" => argument
                .and_then(|a| self.on_signal.get(a))
                .or(self.on_signal.get("")),
            "ONSTARTDRAGGING" => self.on_start_dragging.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    parent: Arc<CnvObject>,

    state: RefCell<ButtonState>,
    event_handlers: ButtonEventHandlers,

    drag: bool,
    sound_normal: Option<String>,
    sound_on_hover: Option<String>,
    sound_on_click: Option<String>,
}

impl Button {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: ButtonProperties) -> Self {
        let is_enabled = props.enable.unwrap_or(true);
        Self {
            parent,
            state: RefCell::new(ButtonState {
                is_enabled,
                is_accented: props.accent.unwrap_or_default(),
                is_draggable: props.draggable.unwrap_or_default(),
                graphics_normal: props.gfx_standard,
                graphics_on_hover: props.gfx_on_move,
                graphics_on_click: props.gfx_on_click,
                priority: props.priority.unwrap_or_default() as isize,
                rect: props.rect,
                is_visible: is_enabled,
            }),
            event_handlers: ButtonEventHandlers {
                on_action: props.on_action,
                on_clicked: props.on_clicked,
                on_done: props.on_done,
                on_dragging: props.on_dragging,
                on_end_dragging: props.on_end_dragging,
                on_focus_off: props.on_focus_off,
                on_focus_on: props.on_focus_on,
                on_init: props.on_init,
                on_paused: props.on_paused,
                on_released: props.on_released,
                on_signal: props.on_signal,
                on_start_dragging: props.on_start_dragging,
            },
            drag: props.drag.unwrap_or_default(),
            sound_normal: props.snd_standard,
            sound_on_hover: props.snd_on_move,
            sound_on_click: props.snd_on_click,
        }
    }

    // custom

    pub fn update(&self, _context: RunnerContext) -> RunnerResult<()> {
        Ok(())
    }
}

impl CnvType for Button {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BUTTON"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("ACCENT") => self.state.borrow_mut().accent().map(|_| None),
            CallableIdentifier::Method("DISABLE") => {
                self.state.borrow_mut().disable().map(|_| None)
            }
            CallableIdentifier::Method("DISABLEBUTVISIBLE") => {
                self.state.borrow_mut().disable_but_visible().map(|_| None)
            }
            CallableIdentifier::Method("DISABLEDRAGGING") => {
                self.state.borrow_mut().disable_dragging().map(|_| None)
            }
            CallableIdentifier::Method("ENABLE") => self.state.borrow_mut().enable().map(|_| None),
            CallableIdentifier::Method("ENABLEDRAGGING") => {
                self.state.borrow_mut().enable_dragging().map(|_| None)
            }
            CallableIdentifier::Method("GETONCLICK") => self.state.borrow().get_on_click(),
            CallableIdentifier::Method("GETONMOVE") => self.state.borrow().get_on_move(),
            CallableIdentifier::Method("GETPRIORITY") => self
                .state
                .borrow()
                .get_priority()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETSTD") => self.state.borrow().get_std(),
            CallableIdentifier::Method("SETONCLICK") => {
                self.state.borrow_mut().set_on_click().map(|_| None)
            }
            CallableIdentifier::Method("SETONMOVE") => {
                self.state.borrow_mut().set_on_move().map(|_| None)
            }
            CallableIdentifier::Method("SETPRIORITY") => {
                self.state.borrow_mut().set_priority().map(|_| None)
            }
            CallableIdentifier::Method("SETRECT") => {
                self.state.borrow_mut().set_rect().map(|_| None)
            }
            CallableIdentifier::Method("SETSTD") => self.state.borrow_mut().set_std().map(|_| None),
            CallableIdentifier::Method("SYN") => self.state.borrow_mut().syn().map(|_| None),
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
        let accent = properties
            .remove("ACCENT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let drag = properties
            .remove("DRAG")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let draggable = properties
            .remove("DRAGGABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let enable = properties
            .remove("ENABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let gfx_on_click = properties.remove("GFXONCLICK").and_then(discard_if_empty);
        let gfx_on_move = properties.remove("GFXONMOVE").and_then(discard_if_empty);
        let gfx_standard = properties.remove("GFXSTANDARD").and_then(discard_if_empty);
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let rect = properties
            .remove("RECT")
            .and_then(discard_if_empty)
            .map(parse_rect)
            .transpose()?;
        let snd_on_click = properties.remove("SNDONCLICK").and_then(discard_if_empty);
        let snd_on_move = properties.remove("SNDONMOVE").and_then(discard_if_empty);
        let snd_standard = properties.remove("SNDSTANDARD").and_then(discard_if_empty);
        let on_action = properties
            .remove("ONACTION")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_clicked = properties
            .remove("ONCLICKED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_dragging = properties
            .remove("ONDRAGGING")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_end_dragging = properties
            .remove("ONENDDRAGGING")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_focus_off = properties
            .remove("ONFOCUSOFF")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_focus_on = properties
            .remove("ONFOCUSON")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_paused = properties
            .remove("ONPAUSED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_released = properties
            .remove("ONRELEASED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_signal = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSIGNAL" {
                on_signal.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONSIGNAL^") {
                on_signal.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let on_start_dragging = properties
            .remove("ONSTARTDRAGGING")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::Button(Button::from_initial_properties(
            parent,
            ButtonProperties {
                accent,
                drag,
                draggable,
                enable,
                gfx_on_click,
                gfx_on_move,
                gfx_standard,
                priority,
                rect,
                snd_on_click,
                snd_on_move,
                snd_standard,
                on_action,
                on_clicked,
                on_done,
                on_dragging,
                on_end_dragging,
                on_focus_off,
                on_focus_on,
                on_init,
                on_paused,
                on_released,
                on_signal,
                on_start_dragging,
            },
        )))
    }
}

impl Initable for Button {
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

impl ButtonState {
    pub fn accent(&mut self) -> RunnerResult<()> {
        // ACCENT
        todo!()
    }

    pub fn disable(&mut self) -> RunnerResult<()> {
        // DISABLE
        todo!()
    }

    pub fn disable_but_visible(&mut self) -> RunnerResult<()> {
        // DISABLEBUTVISIBLE
        todo!()
    }

    pub fn disable_dragging(&mut self) -> RunnerResult<()> {
        // DISABLEDRAGGING
        todo!()
    }

    pub fn enable(&mut self) -> RunnerResult<()> {
        // ENABLE
        todo!()
    }

    pub fn enable_dragging(&mut self) -> RunnerResult<()> {
        // ENABLEDRAGGING
        todo!()
    }

    pub fn get_on_click(&self) -> RunnerResult<Option<CnvValue>> {
        // GETONCLICK
        todo!()
    }

    pub fn get_on_move(&self) -> RunnerResult<Option<CnvValue>> {
        // GETONMOVE
        todo!()
    }

    pub fn get_priority(&self) -> RunnerResult<isize> {
        // GETPRIORITY
        todo!()
    }

    pub fn get_std(&self) -> RunnerResult<Option<CnvValue>> {
        // GETSTD
        todo!()
    }

    pub fn set_on_click(&mut self) -> RunnerResult<()> {
        // SETONCLICK
        todo!()
    }

    pub fn set_on_move(&mut self) -> RunnerResult<()> {
        // SETONMOVE
        todo!()
    }

    pub fn set_priority(&mut self) -> RunnerResult<()> {
        // SETPRIORITY
        todo!()
    }

    pub fn set_rect(&mut self) -> RunnerResult<()> {
        // SETRECT
        todo!()
    }

    pub fn set_std(&mut self) -> RunnerResult<()> {
        // SETSTD
        todo!()
    }

    pub fn syn(&mut self) -> RunnerResult<()> {
        // SYN
        todo!()
    }
}
