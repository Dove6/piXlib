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
pub struct TextProperties {
    // TEXT
    pub font: Option<FontName>,                // FONT
    pub horizontal_justify: Option<bool>,      // HJUSTIFY
    pub hypertext: Option<bool>,               // HYPERTEXT
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    pub priority: Option<i32>,                 // PRIORITY
    pub rect: Option<ReferenceRect>,           // RECT
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
    pub rect: Option<ReferenceRect>,
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

impl EventHandler for TextEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONCOLLISION" => self.on_collision.as_ref(),
            "ONCOLLISIONFINISHED" => self.on_collision_finished.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
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
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("CLEARCLIPPING") => self
                .state
                .borrow_mut()
                .clear_clipping()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("DRAWONTO") => {
                self.state.borrow_mut().draw_onto().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETHEIGHT") => self
                .state
                .borrow()
                .get_height()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETNUMWORDS") => self
                .state
                .borrow()
                .get_num_words()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETPOSITIONX") => self
                .state
                .borrow()
                .get_position_x()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETPOSITIONY") => self
                .state
                .borrow()
                .get_position_y()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETWIDTH") => self
                .state
                .borrow()
                .get_width()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETWORDAT") => {
                self.state.borrow().get_word_at().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETWORDATXY") => {
                self.state.borrow().get_word_at_xy().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETWORDPOSX") => self
                .state
                .borrow()
                .get_word_pos_x()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETWORDPOSY") => self
                .state
                .borrow()
                .get_word_pos_y()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETWORDWIDTH") => self
                .state
                .borrow()
                .get_word_width()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("HIDE") => {
                self.state.borrow_mut().hide().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("INVALIDATE") => {
                self.state.borrow_mut().invalidate().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISNEAR") => {
                self.state.borrow().is_near().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("LOAD") => {
                self.state.borrow_mut().load().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MOVE") => {
                self.state.borrow_mut().move_by().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SEARCH") => {
                self.state.borrow_mut().search().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETCLIPPING") => self
                .state
                .borrow_mut()
                .set_clipping()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETCOLOR") => {
                self.state.borrow_mut().set_color().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETFONT") => {
                self.state.borrow_mut().set_font().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETJUSTIFY") => self
                .state
                .borrow_mut()
                .set_justify()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETOPACITY") => self
                .state
                .borrow_mut()
                .set_opacity()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETPOSITION") => self
                .state
                .borrow_mut()
                .set_position()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETPRIORITY") => self
                .state
                .borrow_mut()
                .set_priority(arguments[0].to_int() as isize)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETRECT") => {
                self.state.borrow_mut().set_rect().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETTEXT") => self
                .state
                .borrow_mut()
                .set_text(arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETTEXTDOUBLE") => self
                .state
                .borrow_mut()
                .set_text_double()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETWORDCOLOR") => self
                .state
                .borrow_mut()
                .set_word_color()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SHOW") => {
                self.state.borrow_mut().show().map(|_| CnvValue::Null)
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
            .map(parse_event_handler)
            .transpose()?;
        let on_collision_finished = properties
            .remove("ONCOLLISIONFINISHED")
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
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
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

impl TextState {
    pub fn clear_clipping(&mut self) -> anyhow::Result<()> {
        // CLEARCLIPPING
        todo!()
    }

    pub fn draw_onto(&mut self) -> anyhow::Result<()> {
        // DRAWONTO
        todo!()
    }

    pub fn get_height(&self) -> anyhow::Result<usize> {
        // GETHEIGHT
        todo!()
    }

    pub fn get_num_words(&self) -> anyhow::Result<usize> {
        // GETNUMWORDS
        todo!()
    }

    pub fn get_position_x(&self) -> anyhow::Result<isize> {
        // GETPOSITIONX
        todo!()
    }

    pub fn get_position_y(&self) -> anyhow::Result<isize> {
        // GETPOSITIONY
        todo!()
    }

    pub fn get_width(&self) -> anyhow::Result<usize> {
        // GETWIDTH
        todo!()
    }

    pub fn get_word_at(&self) -> anyhow::Result<String> {
        // GETWORDAT
        todo!()
    }

    pub fn get_word_at_xy(&self) -> anyhow::Result<String> {
        // GETWORDATXY
        todo!()
    }

    pub fn get_word_pos_x(&self) -> anyhow::Result<isize> {
        // GETWORDPOSX
        todo!()
    }

    pub fn get_word_pos_y(&self) -> anyhow::Result<isize> {
        // GETWORDPOSY
        todo!()
    }

    pub fn get_word_width(&self) -> anyhow::Result<usize> {
        // GETWORDWIDTH
        todo!()
    }

    pub fn hide(&mut self) -> anyhow::Result<()> {
        // HIDE
        todo!()
    }

    pub fn invalidate(&mut self) -> anyhow::Result<()> {
        // INVALIDATE
        todo!()
    }

    pub fn is_near(&self) -> anyhow::Result<bool> {
        // ISNEAR
        todo!()
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        // LOAD
        todo!()
    }

    pub fn move_by(&mut self) -> anyhow::Result<()> {
        // MOVE
        todo!()
    }

    pub fn search(&mut self) -> anyhow::Result<()> {
        // SEARCH
        todo!()
    }

    pub fn set_clipping(&mut self) -> anyhow::Result<()> {
        // SETCLIPPING
        todo!()
    }

    pub fn set_color(&mut self) -> anyhow::Result<()> {
        // SETCOLOR
        todo!()
    }

    pub fn set_font(&mut self) -> anyhow::Result<()> {
        // SETFONT
        todo!()
    }

    pub fn set_justify(&mut self) -> anyhow::Result<()> {
        // SETJUSTIFY
        todo!()
    }

    pub fn set_opacity(&mut self) -> anyhow::Result<()> {
        // SETOPACITY
        todo!()
    }

    pub fn set_position(&mut self) -> anyhow::Result<()> {
        // SETPOSITION
        todo!()
    }

    pub fn set_priority(&mut self, priority: isize) -> anyhow::Result<()> {
        // SETPRIORITY
        self.priority = priority;
        Ok(())
    }

    pub fn set_rect(&mut self) -> anyhow::Result<()> {
        // SETRECT
        todo!()
    }

    pub fn set_text(&mut self, text: String) -> anyhow::Result<()> {
        // SETTEXT
        self.text = text;
        Ok(())
    }

    pub fn set_text_double(&mut self) -> anyhow::Result<()> {
        // SETTEXTDOUBLE
        todo!()
    }

    pub fn set_word_color(&mut self) -> anyhow::Result<()> {
        // SETWORDCOLOR
        todo!()
    }

    pub fn show(&mut self) -> anyhow::Result<()> {
        // SHOW
        todo!()
    }
}
