use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::parsers::{discard_if_empty, parse_i32};

use crate::parser::ast::ParsedScript;

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct MultiArrayProperties {
    // MULTIARRAY
    dimensions: i32, // DIMENSIONS
}

#[derive(Debug, Clone, Default)]
struct MultiArrayState {
    // deduces from methods
    pub values: Vec<CnvValue>,
}

#[derive(Debug, Clone)]
pub struct MultiArrayEventHandlers {}

impl EventHandler for MultiArrayEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct MultiArray {
    parent: Arc<CnvObject>,

    state: RefCell<MultiArrayState>,
    event_handlers: MultiArrayEventHandlers,

    dimension_count: usize,
}

impl MultiArray {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: MultiArrayProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(MultiArrayState {
                ..Default::default()
            }),
            event_handlers: MultiArrayEventHandlers {},
            dimension_count: props.dimensions as usize,
        }
    }
}

impl CnvType for MultiArray {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "MULTIARRAY"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("COUNT") => self.state.borrow_mut().count().map(|_| None),
            CallableIdentifier::Method("LOAD") => self.state.borrow_mut().load().map(|_| None),
            CallableIdentifier::Method("GET") => self.state.borrow().get(),
            CallableIdentifier::Method("GETSIZE") => self
                .state
                .borrow()
                .get_size()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("SAFEGET") => self.state.borrow().safe_get(),
            CallableIdentifier::Method("SAVE") => self.state.borrow_mut().save().map(|_| None),
            CallableIdentifier::Method("SET") => self.state.borrow_mut().set().map(|_| None),
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
        let dimensions = properties
            .remove("DIMENSIONS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?
            .ok_or(TypeParsingError::MissingDimensionCount)?;
        Ok(CnvContent::MultiArray(Self::from_initial_properties(
            parent,
            MultiArrayProperties { dimensions },
        )))
    }
}

impl MultiArrayState {
    pub fn count(&mut self) -> RunnerResult<()> {
        // COUNT
        todo!()
    }

    pub fn load(&mut self) -> RunnerResult<()> {
        // LOAD
        todo!()
    }

    pub fn get(&self) -> RunnerResult<Option<CnvValue>> {
        // GET
        todo!()
    }

    pub fn get_size(&self) -> RunnerResult<usize> {
        // GETSIZE
        todo!()
    }

    pub fn safe_get(&self) -> RunnerResult<Option<CnvValue>> {
        // SAFEGET
        todo!()
    }

    pub fn save(&mut self) -> RunnerResult<()> {
        // SAVE
        todo!()
    }

    pub fn set(&mut self) -> RunnerResult<()> {
        // SET
        todo!()
    }
}