use std::any::Any;

use parsers::discard_if_empty;

use super::*;

#[derive(Debug, Clone)]
pub struct CnvLoaderInit {
    // CNVLOADER
    cnv_loader: Option<String>, // CNVLOADER
}

#[derive(Debug, Clone)]
pub struct CnvLoader {
    parent: Arc<CnvObject>,
    initial_properties: CnvLoaderInit,
}

impl CnvLoader {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: CnvLoaderInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn load() {
        // LOAD
        todo!()
    }

    pub fn release() {
        // RELEASE
        todo!()
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
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let cnv_loader = properties.remove("CNVLOADER").and_then(discard_if_empty);
        Ok(Self::from_initial_properties(
            parent,
            CnvLoaderInit { cnv_loader },
        ))
    }
}
