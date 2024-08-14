use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::parsers::discard_if_empty;

use crate::parser::ast::ParsedScript;

use super::super::common::*;
use super::super::*;
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

impl EventHandler for MusicEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new_content(
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
