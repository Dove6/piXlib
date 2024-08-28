use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{
    discard_if_empty, parse_bool, parse_event_handler, parse_i32, parse_rect, ReferenceRect,
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
    pub rect: Option<ReferenceRect>,     // RECT
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Interaction {
    Hidden,
    #[default]
    None,
    Hovering,
    Pressing,
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
    pub rect: Option<ReferenceRect>,

    pub current_interaction: Interaction,
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
                ..Default::default()
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
}

impl GeneralButton for Button {
    fn is_enabled(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().is_enabled)
    }

    fn get_rect(&self) -> anyhow::Result<Option<Rect>> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow().get_rect(context)
    }

    fn get_priority(&self) -> anyhow::Result<isize> {
        self.state.borrow().get_priority()
    }

    fn handle_lmb_pressed(&self) -> anyhow::Result<()> {
        self.state.borrow_mut().try_set_interaction(
            RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent),
            Interaction::Pressing,
        )
    }

    fn handle_lmb_released(&self) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|internal_events| {
                internal_events.push_back(InternalEvent {
                    context: context.clone(),
                    callable: CallableIdentifier::Event("ONACTION").to_owned(),
                })
            });
        self.state.borrow_mut().try_set_interaction(
            RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent),
            Interaction::Hovering,
        )
    }

    fn handle_cursor_over(&self) -> anyhow::Result<()> {
        self.state
            .borrow_mut()
            .promote_to_hovering_or_keep_pressing(RunnerContext::new_minimal(
                &self.parent.parent.runner,
                &self.parent,
            ))
    }

    fn handle_cursor_away(&self) -> anyhow::Result<()> {
        self.state.borrow_mut().try_set_interaction(
            RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent),
            Interaction::None,
        )
    }

    fn makes_cursor_pointer(&self) -> anyhow::Result<bool> {
        Ok(true)
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
    ) -> anyhow::Result<CnvValue> {
        // log::trace!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("ACCENT") => {
                self.state.borrow_mut().accent().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DISABLE") => self
                .state
                .borrow_mut()
                .disable(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("DISABLEBUTVISIBLE") => self
                .state
                .borrow_mut()
                .disable_but_visible(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("DISABLEDRAGGING") => self
                .state
                .borrow_mut()
                .disable_dragging()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("ENABLE") => self
                .state
                .borrow_mut()
                .enable(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("ENABLEDRAGGING") => self
                .state
                .borrow_mut()
                .enable_dragging()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETONCLICK") => self
                .state
                .borrow()
                .get_on_click()
                .map(|v| v.map(CnvValue::String).unwrap_or_default()),
            CallableIdentifier::Method("GETONMOVE") => self
                .state
                .borrow()
                .get_on_move()
                .map(|v| v.map(CnvValue::String).unwrap_or_default()),
            CallableIdentifier::Method("GETPRIORITY") => self
                .state
                .borrow()
                .get_priority()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETSTD") => self
                .state
                .borrow()
                .get_std()
                .map(|v| v.map(CnvValue::String).unwrap_or_default()),
            CallableIdentifier::Method("SETONCLICK") => self
                .state
                .borrow_mut()
                .set_on_click(&arguments[0].to_string())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETONMOVE") => self
                .state
                .borrow_mut()
                .set_on_move(&arguments[0].to_string())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETPRIORITY") => self
                .state
                .borrow_mut()
                .set_priority(arguments[0].to_int() as isize)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETRECT") => {
                let rect = parse_rect(arguments[0].to_str())?;
                self.state
                    .borrow_mut()
                    .set_rect(rect)
                    .map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETSTD") => self
                .state
                .borrow_mut()
                .set_std(&arguments[0].to_string())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SYN") => {
                self.state.borrow_mut().syn().map(|_| CnvValue::Null)
            }
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
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        self.state
            .borrow_mut()
            .use_and_drop_mut(|state| -> anyhow::Result<()> {
                state.set_interaction(context.clone(), Interaction::Hidden)?;
                if state.is_enabled {
                    state.set_interaction(context.clone(), Interaction::None)?;
                }
                Ok(())
            })?;
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

impl ButtonState {
    pub fn accent(&mut self) -> anyhow::Result<()> {
        // ACCENT
        todo!()
    }

    pub fn disable(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // DISABLE
        self.is_enabled = false;
        self.set_interaction(context, Interaction::Hidden)
    }

    pub fn disable_but_visible(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // DISABLEBUTVISIBLE
        self.is_enabled = false;
        self.set_interaction(context, Interaction::None)
    }

    pub fn disable_dragging(&mut self) -> anyhow::Result<()> {
        // DISABLEDRAGGING
        todo!()
    }

    pub fn enable(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // ENABLE
        if self.is_enabled {
            return Ok(());
        }
        self.is_enabled = true;
        self.set_interaction(context, Interaction::None)
    }

    pub fn enable_dragging(&mut self) -> anyhow::Result<()> {
        // ENABLEDRAGGING
        todo!()
    }

    pub fn get_on_click(&self) -> anyhow::Result<Option<String>> {
        // GETONCLICK
        Ok(self.graphics_on_click.clone())
    }

    pub fn get_on_move(&self) -> anyhow::Result<Option<String>> {
        // GETONMOVE
        Ok(self.graphics_on_hover.clone())
    }

    pub fn get_priority(&self) -> anyhow::Result<isize> {
        // GETPRIORITY
        Ok(self.priority)
    }

    pub fn get_std(&self) -> anyhow::Result<Option<String>> {
        // GETSTD
        Ok(self.graphics_normal.clone())
    }

    pub fn set_on_click(&mut self, object_name: &str) -> anyhow::Result<()> {
        // SETONCLICK
        self.graphics_on_click = Some(object_name.to_owned());
        Ok(())
    }

    pub fn set_on_move(&mut self, object_name: &str) -> anyhow::Result<()> {
        // SETONMOVE
        self.graphics_on_hover = Some(object_name.to_owned());
        Ok(())
    }

    pub fn set_priority(&mut self, priority: isize) -> anyhow::Result<()> {
        // SETPRIORITY
        self.priority = priority;
        Ok(())
    }

    pub fn set_rect(&mut self, rect: ReferenceRect) -> anyhow::Result<()> {
        // SETRECT
        self.rect = Some(rect);
        Ok(())
    }

    pub fn set_std(&mut self, object_name: &str) -> anyhow::Result<()> {
        // SETSTD
        self.graphics_normal = Some(object_name.to_owned());
        Ok(())
    }

    pub fn syn(&mut self) -> anyhow::Result<()> {
        // SYN
        todo!()
    }

    // custom

    pub fn get_rect(&self, context: RunnerContext) -> anyhow::Result<Option<Rect>> {
        if let Some(reference_rect) = &self.rect {
            match reference_rect {
                ReferenceRect::Literal(rect) => Ok(Some(*rect)),
                ReferenceRect::Reference(reference) => {
                    let object = context.runner.get_object(reference).ok_or(
                        RunnerError::ObjectNotFound {
                            name: reference.clone(),
                        },
                    )?;
                    let graphics: &dyn GeneralGraphics = match &object.content {
                        CnvContent::Animation(a) => a,
                        CnvContent::Image(i) => i,
                        _ => return Err(RunnerError::ExpectedGraphicsObject.into()),
                    };
                    graphics.get_rect()
                }
            }
        } else if let Some(graphics_normal) = &self.graphics_normal {
            let object =
                context
                    .runner
                    .get_object(graphics_normal)
                    .ok_or(RunnerError::ObjectNotFound {
                        name: graphics_normal.clone(),
                    })?;
            let graphics: &dyn GeneralGraphics = match &object.content {
                CnvContent::Animation(a) => a,
                CnvContent::Image(i) => i,
                _ => return Err(RunnerError::ExpectedGraphicsObject.into()),
            };
            graphics.get_rect()
        } else {
            Ok(None)
        }
    }

    fn set_interaction(
        &mut self,
        context: RunnerContext,
        mut interaction: Interaction,
    ) -> anyhow::Result<()> {
        // log::trace!(
        //     "{}.set_interaction({:?})",
        //     context.current_object.name, interaction
        // );
        if interaction == self.current_interaction {
            return Ok(());
        }
        let CnvContent::Button(button) = &context.current_object.content else {
            panic!();
        };
        let prev_interaction = self.current_interaction;
        self.current_interaction = interaction;
        if interaction == Interaction::Pressing && self.graphics_on_click.is_none() {
            interaction = Interaction::Hovering;
        }
        if interaction == Interaction::Hovering && self.graphics_on_hover.is_none() {
            interaction = Interaction::None;
        }
        if let Some(normal_obj) = self
            .graphics_normal
            .as_ref()
            .and_then(|name| context.runner.get_object(name))
        {
            let normal_graphics: &dyn GeneralGraphics = match &normal_obj.content {
                CnvContent::Animation(a) => a,
                CnvContent::Image(i) => i,
                _ => return Err(RunnerError::ExpectedGraphicsObject.into()),
            };
            if interaction == Interaction::None {
                normal_graphics.show()
            } else {
                normal_graphics.hide()
            }?
        } /*else {
            log::trace!(
                "Normal sprite not found for button {}",
                context.current_object.name
            );
        }*/;
        if let Some(normal_sound_obj) = button
            .sound_normal
            .as_ref()
            .and_then(|name| context.runner.get_object(name))
        {
            let CnvContent::Sound(normal_sound) = &normal_sound_obj.content else {
                return Err(RunnerError::ExpectedSoundObject.into());
            };
            if interaction == Interaction::None {
                normal_sound.play()
            } else {
                normal_sound.stop()
            }?
        }
        if let Some(on_hover_obj) = self
            .graphics_on_hover
            .as_ref()
            .and_then(|name| context.runner.get_object(name))
        {
            let on_hover_graphics: &dyn GeneralGraphics = match &on_hover_obj.content {
                CnvContent::Animation(a) => a,
                CnvContent::Image(i) => i,
                _ => return Err(RunnerError::ExpectedGraphicsObject.into()),
            };
            if interaction == Interaction::Hovering {
                on_hover_graphics.show()
            } else {
                on_hover_graphics.hide()
            }?
        } /*else {
            log::trace!(
                "Hovering sprite not found for button {}",
                context.current_object.name
            );
        }*/;
        if let Some(on_hover_sound_obj) = button
            .sound_on_hover
            .as_ref()
            .and_then(|name| context.runner.get_object(name))
        {
            let CnvContent::Sound(on_hover_sound) = &on_hover_sound_obj.content else {
                return Err(RunnerError::ExpectedSoundObject.into());
            };
            if interaction == Interaction::Hovering {
                on_hover_sound.play()
            } else {
                on_hover_sound.stop()
            }?
        }
        if let Some(on_click_obj) = self
            .graphics_on_click
            .as_ref()
            .and_then(|name| context.runner.get_object(name))
        {
            let on_click_graphics: &dyn GeneralGraphics = match &on_click_obj.content {
                CnvContent::Animation(a) => a,
                CnvContent::Image(i) => i,
                _ => return Err(RunnerError::ExpectedGraphicsObject.into()),
            };
            if interaction == Interaction::Pressing {
                on_click_graphics.show()
            } else {
                on_click_graphics.hide()
            }?
        } /*else {
            log::trace!(
                "Pressing sprite not found for button {}",
                context.current_object.name
            );
        }*/;
        if let Some(on_click_sound_obj) = button
            .sound_on_click
            .as_ref()
            .and_then(|name| context.runner.get_object(name))
        {
            let CnvContent::Sound(on_click_sound) = &on_click_sound_obj.content else {
                return Err(RunnerError::ExpectedSoundObject.into());
            };
            if interaction == Interaction::Pressing {
                on_click_sound.play()
            } else {
                on_click_sound.stop()
            }?
        }
        if prev_interaction == Interaction::None {
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        context: context.clone().with_arguments(Vec::new()),
                        callable: CallableIdentifier::Event("ONFOCUSON").to_owned(),
                    })
                });
        } else if interaction == Interaction::None {
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        context: context.clone().with_arguments(Vec::new()),
                        callable: CallableIdentifier::Event("ONFOCUSOFF").to_owned(),
                    })
                });
        }
        if prev_interaction == Interaction::Pressing {
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        context: context.clone().with_arguments(Vec::new()),
                        callable: CallableIdentifier::Event("RELEASED").to_owned(),
                    })
                });
        } else if interaction == Interaction::Pressing {
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        context: context.clone().with_arguments(Vec::new()),
                        callable: CallableIdentifier::Event("CLICKED").to_owned(),
                    })
                });
        }
        Ok(())
    }

    pub fn try_set_interaction(
        &mut self,
        context: RunnerContext,
        interaction: Interaction,
    ) -> anyhow::Result<()> {
        if !self.is_enabled {
            return Ok(());
        }
        if self.current_interaction == Interaction::Hidden || interaction == Interaction::Hidden {
            return Ok(());
        }
        self.set_interaction(context, interaction)
    }

    pub fn promote_to_hovering_or_keep_pressing(
        &mut self,
        context: RunnerContext,
    ) -> anyhow::Result<()> {
        if matches!(
            self.current_interaction,
            Interaction::Pressing | Interaction::Hovering
        ) {
            return Ok(());
        }
        self.try_set_interaction(context, Interaction::Hovering)
    }
}
