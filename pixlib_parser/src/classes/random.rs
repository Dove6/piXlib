use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct RandomInit {
    // RAND
}

#[derive(Debug, Clone)]
pub struct Random {
    parent: Arc<CnvObject>,
    initial_properties: RandomInit,
}

impl Random {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: RandomInit) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn get() {
        // GET
        todo!()
    }

    pub fn get_plenty() {
        // GETPLENTY
        todo!()
    }
}

impl CnvType for Random {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "RANDOM"
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
        &mut self,
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
        _properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        Ok(Self::from_initial_properties(parent, RandomInit {}))
    }
}
