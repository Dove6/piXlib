use std::{any::Any, cell::RefCell};

use content::EventHandler;
use parsers::discard_if_empty;

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct CnvLoaderProperties {
    // CNVLOADER
    cnv_loader: Option<String>, // CNVLOADER
}

#[derive(Debug, Clone, Default)]
struct CnvLoaderState {}

#[derive(Debug, Clone)]
pub struct CnvLoaderEventHandlers {}

impl EventHandler for CnvLoaderEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct CnvLoader {
    parent: Arc<CnvObject>,

    state: RefCell<CnvLoaderState>,
    event_handlers: CnvLoaderEventHandlers,

    cnv_loader: String,
}

impl CnvLoader {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: CnvLoaderProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(CnvLoaderState {
                ..Default::default()
            }),
            event_handlers: CnvLoaderEventHandlers {},
            cnv_loader: props.cnv_loader.unwrap_or_default(),
        }
    }
}

impl CnvType for CnvLoader {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "CNVLOADER"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("LOAD") => self.state.borrow_mut().load().map(|_| None),
            CallableIdentifier::Method("RELEASE") => {
                self.state.borrow_mut().release().map(|_| None)
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self.event_handlers.get(
                    event_name,
                    arguments.get(0).map(|v| v.to_string()).as_deref(),
                ) {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let cnv_loader = properties.remove("CNVLOADER").and_then(discard_if_empty);
        Ok(CnvContent::CnvLoader(Self::from_initial_properties(
            parent,
            CnvLoaderProperties { cnv_loader },
        )))
    }
}

impl CnvLoaderState {
    pub fn load(&mut self) -> RunnerResult<()> {
        // LOAD
        todo!()
    }

    pub fn release(&mut self) -> RunnerResult<()> {
        // RELEASE
        todo!()
    }
}
