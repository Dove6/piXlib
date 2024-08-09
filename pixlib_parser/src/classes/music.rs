use std::{any::Any, cell::RefCell};

use parsers::discard_if_empty;

use super::*;

#[derive(Debug, Clone)]
pub struct MusicProperties {
    // MUSIC
    filename: Option<String>, // FILENAME
}

#[derive(Debug, Clone, Default)]
struct MusicState {
    file_data: SoundFileData,
}

#[derive(Debug, Clone)]
pub struct MusicEventHandlers {}

#[derive(Debug, Clone)]
pub struct Music {
    parent: Arc<CnvObject>,

    state: RefCell<MusicState>,
    event_handlers: MusicEventHandlers,
}

impl Music {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: MusicProperties) -> Self {
        let music = Self {
            parent,
            state: RefCell::new(MusicState {
                ..Default::default()
            }),
            event_handlers: MusicEventHandlers {},
        };
        if let Some(filename) = props.filename {
            music.state.borrow_mut().file_data = SoundFileData::NotLoaded(filename);
        }
        music
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
        &self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
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
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        Ok(CnvContent::Music(Self::from_initial_properties(
            parent,
            MusicProperties { filename },
        )))
    }
}
