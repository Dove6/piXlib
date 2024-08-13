use std::{any::Any, cell::RefCell};

use initable::Initable;
use parsers::{discard_if_empty, parse_bool, parse_i32, parse_program, parse_rect, Rect};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

use super::*;

#[derive(Debug, Clone)]
pub struct TextProperties {
    // TEXT
    pub font: Option<FontName>,                // FONT
    pub horizontal_justify: Option<bool>,      // HJUSTIFY
    pub hypertext: Option<bool>,               // HYPERTEXT
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    pub priority: Option<i32>,                 // PRIORITY
    pub rect: Option<Rect>,                    // RECT
    pub text: Option<String>,                  // TEXT
    pub to_canvas: Option<bool>,               // TOCANVAS
    pub visible: Option<bool>,                 // VISIBLE
    pub vertical_justify: Option<bool>,        // VJUSTIFY

    pub on_collision: Option<Arc<ParsedScript>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<ParsedScript>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<ParsedScript>>,      // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,      // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>,    // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct TextState {
    // initialized from properties
    pub font: Option<FontName>,
    pub is_justified_horizontally: bool,
    pub does_monitor_collision: bool,
    pub priority: isize,
    pub rect: Option<Rect>,
    pub text: String,
    pub is_visible: bool,
    pub is_justified_vertically: bool,

    // deduced from methods
    pub opacity: usize,
    pub color: Option<String>,
    pub clipping: String,
}

#[derive(Debug, Clone)]
pub struct TextEventHandlers {
    pub on_collision: Option<Arc<ParsedScript>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<ParsedScript>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<ParsedScript>>,      // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,      // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>,    // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Text {
    parent: Arc<CnvObject>,

    state: RefCell<TextState>,
    event_handlers: TextEventHandlers,

    should_collisions_respect_alpha: bool,
    should_draw_to_canvas: bool,
}

impl Text {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: TextProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(TextState {
                font: props.font,
                is_justified_horizontally: props.horizontal_justify.unwrap_or_default(),
                does_monitor_collision: props.monitor_collision.unwrap_or_default(),
                priority: props.priority.unwrap_or_default() as isize,
                rect: props.rect,
                text: props.text.unwrap_or_default(),
                is_visible: props.visible.unwrap_or(true),
                is_justified_vertically: props.vertical_justify.unwrap_or_default(),
                ..Default::default()
            }),
            event_handlers: TextEventHandlers {
                on_collision: props.on_collision,
                on_collision_finished: props.on_collision_finished,
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
            },
            should_collisions_respect_alpha: props.monitor_collision_alpha.unwrap_or_default(),
            should_draw_to_canvas: props.to_canvas.unwrap_or(true),
        }
    }
}

impl CnvType for Text {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "TEXT"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("CLEARCLIPPING") => {
                self.state.borrow_mut().clear_clipping().map(|_| None)
            }
            CallableIdentifier::Method("DRAWONTO") => {
                self.state.borrow_mut().draw_onto().map(|_| None)
            }
            CallableIdentifier::Method("GETHEIGHT") => self
                .state
                .borrow()
                .get_height()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETNUMWORDS") => self
                .state
                .borrow()
                .get_num_words()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPOSITIONX") => self
                .state
                .borrow()
                .get_position_x()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPOSITIONY") => self
                .state
                .borrow()
                .get_position_y()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETWIDTH") => self
                .state
                .borrow()
                .get_width()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETWORDAT") => self
                .state
                .borrow()
                .get_word_at()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETWORDATXY") => self
                .state
                .borrow()
                .get_word_at_xy()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETWORDPOSX") => self
                .state
                .borrow()
                .get_word_pos_x()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETWORDPOSY") => self
                .state
                .borrow()
                .get_word_pos_y()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETWORDWIDTH") => self
                .state
                .borrow()
                .get_word_width()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("HIDE") => self.state.borrow_mut().hide().map(|_| None),
            CallableIdentifier::Method("INVALIDATE") => {
                self.state.borrow_mut().invalidate().map(|_| None)
            }
            CallableIdentifier::Method("ISNEAR") => self
                .state
                .borrow()
                .is_near()
                .map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Method("LOAD") => self.state.borrow_mut().load().map(|_| None),
            CallableIdentifier::Method("MOVE") => self.state.borrow_mut().move_by().map(|_| None),
            CallableIdentifier::Method("SEARCH") => self.state.borrow_mut().search().map(|_| None),
            CallableIdentifier::Method("SETCLIPPING") => {
                self.state.borrow_mut().set_clipping().map(|_| None)
            }
            CallableIdentifier::Method("SETCOLOR") => {
                self.state.borrow_mut().set_color().map(|_| None)
            }
            CallableIdentifier::Method("SETFONT") => {
                self.state.borrow_mut().set_font().map(|_| None)
            }
            CallableIdentifier::Method("SETJUSTIFY") => {
                self.state.borrow_mut().set_justify().map(|_| None)
            }
            CallableIdentifier::Method("SETOPACITY") => {
                self.state.borrow_mut().set_opacity().map(|_| None)
            }
            CallableIdentifier::Method("SETPOSITION") => {
                self.state.borrow_mut().set_position().map(|_| None)
            }
            CallableIdentifier::Method("SETPRIORITY") => {
                self.state.borrow_mut().set_priority().map(|_| None)
            }
            CallableIdentifier::Method("SETRECT") => {
                self.state.borrow_mut().set_rect().map(|_| None)
            }
            CallableIdentifier::Method("SETTEXT") => {
                self.state.borrow_mut().set_text().map(|_| None)
            }
            CallableIdentifier::Method("SETTEXTDOUBLE") => {
                self.state.borrow_mut().set_text_double().map(|_| None)
            }
            CallableIdentifier::Method("SETWORDCOLOR") => {
                self.state.borrow_mut().set_word_color().map(|_| None)
            }
            CallableIdentifier::Method("SHOW") => self.state.borrow_mut().show().map(|_| None),
            CallableIdentifier::Event("ONCOLLISION") => {
                if let Some(v) = self.event_handlers.on_collision.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONCOLLISIONFINISHED") => {
                if let Some(v) = self.event_handlers.on_collision_finished.as_ref() {
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

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let font = properties.remove("FONT").and_then(discard_if_empty);
        let horizontal_justify = properties
            .remove("HJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let hypertext = properties
            .remove("HYPERTEXT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
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
        let text = properties.remove("TEXT").and_then(discard_if_empty);
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let vertical_justify = properties
            .remove("VJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let on_collision = properties
            .remove("ONCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_collision_finished = properties
            .remove("ONCOLLISIONFINISHED")
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
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(CnvContent::Text(Self::from_initial_properties(
            parent,
            TextProperties {
                font,
                horizontal_justify,
                hypertext,
                monitor_collision,
                monitor_collision_alpha,
                priority,
                rect,
                text,
                to_canvas,
                visible,
                vertical_justify,
                on_collision,
                on_collision_finished,
                on_done,
                on_init,
                on_signal,
            },
        )))
    }
}

impl Initable for Text {
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

impl TextState {
    pub fn clear_clipping(&mut self) -> RunnerResult<()> {
        // CLEARCLIPPING
        todo!()
    }

    pub fn draw_onto(&mut self) -> RunnerResult<()> {
        // DRAWONTO
        todo!()
    }

    pub fn get_height(&self) -> RunnerResult<usize> {
        // GETHEIGHT
        todo!()
    }

    pub fn get_num_words(&self) -> RunnerResult<usize> {
        // GETNUMWORDS
        todo!()
    }

    pub fn get_position_x(&self) -> RunnerResult<isize> {
        // GETPOSITIONX
        todo!()
    }

    pub fn get_position_y(&self) -> RunnerResult<isize> {
        // GETPOSITIONY
        todo!()
    }

    pub fn get_width(&self) -> RunnerResult<usize> {
        // GETWIDTH
        todo!()
    }

    pub fn get_word_at(&self) -> RunnerResult<String> {
        // GETWORDAT
        todo!()
    }

    pub fn get_word_at_xy(&self) -> RunnerResult<String> {
        // GETWORDATXY
        todo!()
    }

    pub fn get_word_pos_x(&self) -> RunnerResult<isize> {
        // GETWORDPOSX
        todo!()
    }

    pub fn get_word_pos_y(&self) -> RunnerResult<isize> {
        // GETWORDPOSY
        todo!()
    }

    pub fn get_word_width(&self) -> RunnerResult<usize> {
        // GETWORDWIDTH
        todo!()
    }

    pub fn hide(&mut self) -> RunnerResult<()> {
        // HIDE
        todo!()
    }

    pub fn invalidate(&mut self) -> RunnerResult<()> {
        // INVALIDATE
        todo!()
    }

    pub fn is_near(&self) -> RunnerResult<bool> {
        // ISNEAR
        todo!()
    }

    pub fn load(&mut self) -> RunnerResult<()> {
        // LOAD
        todo!()
    }

    pub fn move_by(&mut self) -> RunnerResult<()> {
        // MOVE
        todo!()
    }

    pub fn search(&mut self) -> RunnerResult<()> {
        // SEARCH
        todo!()
    }

    pub fn set_clipping(&mut self) -> RunnerResult<()> {
        // SETCLIPPING
        todo!()
    }

    pub fn set_color(&mut self) -> RunnerResult<()> {
        // SETCOLOR
        todo!()
    }

    pub fn set_font(&mut self) -> RunnerResult<()> {
        // SETFONT
        todo!()
    }

    pub fn set_justify(&mut self) -> RunnerResult<()> {
        // SETJUSTIFY
        todo!()
    }

    pub fn set_opacity(&mut self) -> RunnerResult<()> {
        // SETOPACITY
        todo!()
    }

    pub fn set_position(&mut self) -> RunnerResult<()> {
        // SETPOSITION
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

    pub fn set_text(&mut self) -> RunnerResult<()> {
        // SETTEXT
        todo!()
    }

    pub fn set_text_double(&mut self) -> RunnerResult<()> {
        // SETTEXTDOUBLE
        todo!()
    }

    pub fn set_word_color(&mut self) -> RunnerResult<()> {
        // SETWORDCOLOR
        todo!()
    }

    pub fn show(&mut self) -> RunnerResult<()> {
        // SHOW
        todo!()
    }
}
