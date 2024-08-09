use std::{any::Any, cell::RefCell};

use parsers::discard_if_empty;

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

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("LOAD") => self.state.borrow_mut().load().map(|_| None),
            CallableIdentifier::Method("RELEASE") => {
                self.state.borrow_mut().release().map(|_| None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
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
