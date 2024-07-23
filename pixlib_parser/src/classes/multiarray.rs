use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct MultiArrayInit {
    // MULTIARRAY
    dimensions: Option<i32>, // DIMENSIONS
}

#[derive(Debug, Clone)]
pub struct MultiArray {
    initial_properties: MultiArrayInit,
}

impl MultiArray {
    pub fn from_initial_properties(initial_properties: MultiArrayInit) -> Self {
        Self { initial_properties }
    }

    pub fn count() {
        // COUNT
        todo!()
    }

    pub fn load() {
        // LOAD
        todo!()
    }

    pub fn get() {
        // GET
        todo!()
    }

    pub fn get_size() {
        // GETSIZE
        todo!()
    }

    pub fn safe_get() {
        // SAFEGET
        todo!()
    }

    pub fn save() {
        // SAVE
        todo!()
    }

    pub fn set() {
        // SET
        todo!()
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

    fn new(path: Arc<Path>, mut properties: HashMap<String, String>, filesystem: &dyn FileSystem) -> Result<Self, TypeParsingError> {
        let dimensions = properties
            .remove("DIMENSIONS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self::from_initial_properties(MultiArrayInit { dimensions }))
    }
}
