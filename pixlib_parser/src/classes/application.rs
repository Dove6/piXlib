use std::any::Any;

use parsers::{discard_if_empty, parse_comma_separated, parse_datetime};

use super::*;

#[derive(Debug, Clone)]
pub struct ApplicationInit {
    // APPLICATION
    pub author: Option<String>,                  // AUTHOR
    pub bloomoo_version: Option<String>,         // BLOOMOO_VERSION
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub description: Option<String>,             // DESCRIPTION
    pub episodes: Option<Vec<EpisodeName>>,      // EPISODES
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub path: Option<String>,                    // PATH
    pub start_with: Option<EpisodeName>,         // STARTWITH
    pub version: Option<String>,                 // VERSION
}

#[derive(Debug, Clone)]
pub struct Application {
    parent: Arc<CnvObject>,
    initial_properties: ApplicationInit,
}

impl Application {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: ApplicationInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn disable_music(&self) {
        // DISABLEMUSIC
        todo!()
    }

    pub fn enable_music(&self) {
        // ENABLEMUSIC
        todo!()
    }

    pub fn exists_env(&self) {
        // EXISTSENV
        todo!()
    }

    pub fn exit(&self) {
        // EXIT
        todo!()
    }

    pub fn get_language(&self) {
        // GETLANGUAGE
        todo!()
    }

    pub fn get_player(&self) {
        // GETPLAYER
        todo!()
    }

    pub fn goto(&self) {
        // GOTO
        todo!()
    }

    pub fn print(&self) {
        // PRINT
        todo!()
    }

    pub fn reload(&self) {
        // RELOAD
        todo!()
    }

    pub fn restart(&self) {
        // RESTART
        todo!()
    }

    pub fn run(&self) {
        // RUN
        todo!()
    }

    pub fn run_env(&self) {
        // RUNENV
        todo!()
    }

    pub fn set_language(&self) {
        // SETLANGUAGE
        todo!()
    }

    pub fn start_dragging_window(&self) {
        // STARTDRAGGINGWINDOW
        todo!()
    }

    pub fn stop_dragging_window(&self) {
        // STOPDRAGGINGWINDOW
        todo!()
    }

    pub fn store_binary(&self) {
        // STOREBINARY
        todo!()
    }

    ///

    pub fn get_episode_list(&self) -> Vec<String> {
        self.initial_properties
            .episodes
            .clone()
            .unwrap_or(Vec::new())
    }

    pub fn get_script_path(&self) -> Option<String> {
        self.initial_properties.path.clone()
    }
}

impl CnvType for Application {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "APPLICATION"
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
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        todo!()
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "PATH" => self.initial_properties.path.clone().map(|v| v.into()),
            "EPISODES" => self.initial_properties.episodes.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let bloomoo_version = properties
            .remove("BLOOMOO_VERSION")
            .and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let episodes = properties
            .remove("EPISODES")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let start_with = properties.remove("STARTWITH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        Ok(Self::from_initial_properties(
            parent,
            ApplicationInit {
                author,
                bloomoo_version,
                creation_time,
                description,
                episodes,
                last_modify_time,
                path,
                start_with,
                version,
            },
        ))
    }
}
