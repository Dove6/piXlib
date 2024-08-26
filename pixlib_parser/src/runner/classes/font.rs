use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler, FontDef};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct FontProperties {
    // FONT
    pub defs: HashMap<FontDef, Option<String>>,

    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct FontState {
    // deduced from methods
    pub color: String,
    pub family: String,
    pub size: usize,
    pub style: String,
}

#[derive(Debug, Clone)]
pub struct FontEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

impl EventHandler for FontEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    parent: Arc<CnvObject>,

    state: RefCell<FontState>,
    event_handlers: FontEventHandlers,

    font_definitions: HashMap<FontDef, Option<String>>,
}

impl Font {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: FontProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(FontState {
                ..Default::default()
            }),
            event_handlers: FontEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
            },
            font_definitions: props.defs,
        }
    }
}

lazy_static! {
    static ref FONT_DEF_REGEX: Regex = Regex::new(r"^DEF_(\w+)_(\w+)_(\d+)$").unwrap();
}

impl CnvType for Font {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "FONT"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("GETHEIGHT") => self
                .state
                .borrow()
                .get_height()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("SETCOLOR") => {
                self.state.borrow_mut().set_color().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETFAMILY") => {
                self.state.borrow_mut().set_family().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETSIZE") => {
                self.state.borrow_mut().set_size().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETSTYLE") => {
                self.state.borrow_mut().set_style().map(|_| CnvValue::Null)
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
        let defs: HashMap<FontDef, Option<String>> = properties
            .into_iter()
            .filter_map(|(k, v)| {
                FONT_DEF_REGEX.captures(k.as_ref()).map(|m| {
                    (
                        FontDef {
                            family: m[1].to_owned(),
                            style: m[2].to_owned(),
                            size: m[3].parse().unwrap(),
                        },
                        Some(v),
                    )
                })
            })
            .collect();
        Ok(CnvContent::Font(Self::from_initial_properties(
            parent,
            FontProperties {
                defs,
                on_done,
                on_init,
                on_signal,
            },
        )))
    }
}

impl Initable for Font {
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

impl FontState {
    pub fn get_height(&self) -> anyhow::Result<usize> {
        // GETHEIGHT
        todo!()
    }

    pub fn set_color(&mut self) -> anyhow::Result<()> {
        // SETCOLOR
        todo!()
    }

    pub fn set_family(&mut self) -> anyhow::Result<()> {
        // SETFAMILY
        todo!()
    }

    pub fn set_size(&mut self) -> anyhow::Result<()> {
        // SETSIZE
        todo!()
    }

    pub fn set_style(&mut self) -> anyhow::Result<()> {
        // SETSTYLE
        todo!()
    }
}
