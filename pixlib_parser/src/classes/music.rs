use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct MusicInit {
    // MUSIC
    filename: Option<String>, // FILENAME
}

#[derive(Debug, Clone)]
pub struct Music {
    parent: Arc<RwLock<CnvObject>>,
    initial_properties: MusicInit,
}

impl Music {
    pub fn from_initial_properties(
        parent: Arc<RwLock<CnvObject>>,
        initial_properties: MusicInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }
}

impl CnvType for Music {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "MUSIC"
    }

    fn has_event(&self, _name: &str) -> bool {
        false
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
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
        parent: Arc<RwLock<CnvObject>>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        Ok(Self::from_initial_properties(
            parent,
            MusicInit { filename },
        ))
    }
}
