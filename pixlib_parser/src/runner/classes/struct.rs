use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler, STRUCT_FIELDS_REGEX};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct StructProperties {
    // STRUCT
    pub fields: Option<Vec<(String, TypeName)>>,

    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct StructState {
    // deduced from methods
    pub fields: HashMap<String, CnvValue>,
}

#[derive(Debug, Clone)]
pub struct StructEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

impl EventHandler for StructEventHandlers {
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
pub struct Struct {
    parent: Arc<CnvObject>,

    state: RefCell<StructState>,
    event_handlers: StructEventHandlers,

    fields: Vec<(String, TypeName)>,
}

impl Struct {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: StructProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(StructState {
                ..Default::default()
            }),
            event_handlers: StructEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
            },
            fields: props.fields.unwrap_or_default(),
        }
    }
}

impl CnvType for Struct {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "STRUCT"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("GETFIELD") => self
                .state
                .borrow_mut()
                .get_field(&arguments[0].to_str())
                .map(|_| None),
            CallableIdentifier::Method("SET") => self.state.borrow_mut().set().map(|_| None),
            CallableIdentifier::Method("SETFIELD") => self
                .state
                .borrow_mut()
                .set_field(&arguments[0].to_str(), arguments[1].clone())
                .map(|_| None),
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
            }
            .into()),
        }
    }

    fn new_content(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let fields = properties
            .remove("FIELDS")
            .and_then(discard_if_empty)
            .map(|s| {
                s.split(',')
                    .map(|f| {
                        let m = STRUCT_FIELDS_REGEX.captures(f).unwrap();
                        (m[1].to_owned(), m[2].to_owned())
                    })
                    .collect()
            });
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
        Ok(CnvContent::Struct(Self::from_initial_properties(
            parent,
            StructProperties {
                fields,
                on_done,
                on_init,
                on_signal,
            },
        )))
    }
}

impl Initable for Struct {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
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

impl StructState {
    pub fn get_field(&self, _name: &str) -> anyhow::Result<CnvValue> {
        // GETFIELD
        todo!()
    }

    pub fn set(&mut self) -> anyhow::Result<()> {
        // SET
        todo!()
    }

    pub fn set_field(&mut self, _name: &str, _value: CnvValue) -> anyhow::Result<()> {
        // SETFIELD
        todo!()
    }
}
