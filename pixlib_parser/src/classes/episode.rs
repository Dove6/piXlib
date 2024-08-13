use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_comma_separated, parse_datetime};

use super::*;

#[derive(Debug, Clone)]
pub struct EpisodeProperties {
    pub author: Option<String>,                  // AUTHOR
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub description: Option<String>,             // DESCRIPTION
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub path: Option<String>,                    // PATH
    pub scenes: Option<Vec<SceneName>>,          // SCENES
    pub start_with: Option<SceneName>,           // STARTWITH
    pub version: Option<String>,                 // VERSION
}

#[derive(Debug, Clone, Default)]
pub struct EpisodeState {}

#[derive(Debug, Clone)]
pub struct EpisodeEventHandlers {}

#[derive(Debug, Clone)]
pub struct Episode {
    // EPISODE
    parent: Arc<CnvObject>,

    state: RefCell<EpisodeState>,
    event_handlers: EpisodeEventHandlers,

    author: String,
    creation_time: Option<DateTime<Utc>>,
    description: String,
    last_modify_time: Option<DateTime<Utc>>,
    path: Option<String>,
    scenes: Vec<SceneName>,
    start_with: Option<SceneName>,
    version: String,
}

impl Episode {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: EpisodeProperties) -> Self {
        let mut episode = Self {
            parent,
            state: RefCell::new(EpisodeState {
                ..Default::default()
            }),
            event_handlers: EpisodeEventHandlers {},
            author: props.author.unwrap_or_default(),
            creation_time: props.creation_time,
            description: props.description.unwrap_or_default(),
            last_modify_time: props.last_modify_time,
            path: props.path,
            scenes: props.scenes.unwrap_or_default(),
            start_with: props.start_with,
            version: props.version.unwrap_or_default(),
        };
        if episode.start_with.is_none() && !episode.scenes.is_empty() {
            episode.start_with = Some(episode.scenes[0].clone());
        }
        episode
    }

    ///

    pub fn get_script_path(&self) -> Option<String> {
        self.path.clone()
    }

    pub fn get_scene_list(&self) -> Vec<String> {
        self.scenes.clone()
    }
}

impl CnvType for Episode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "EPISODE"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("BACK") => self.state.borrow_mut().back().map(|_| None),
            CallableIdentifier::Method("GETCURRENTSCENE") => {
                self.state.borrow().get_current_scene().map(|_| None)
            }
            CallableIdentifier::Method("GETLATESTSCENE") => {
                self.state.borrow().get_latest_scene().map(|_| None)
            }
            CallableIdentifier::Method("GOTO") => self
                .state
                .borrow_mut()
                .go_to(self, &arguments[0].to_string())
                .map(|_| None),
            CallableIdentifier::Method("NEXT") => self.state.borrow_mut().next().map(|_| None),
            CallableIdentifier::Method("PREV") => self.state.borrow_mut().prev().map(|_| None),
            CallableIdentifier::Method("RESTART") => {
                self.state.borrow_mut().restart().map(|_| None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let scenes = properties
            .remove("SCENES")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let start_with = properties.remove("STARTWITH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        Ok(CnvContent::Episode(Episode::from_initial_properties(
            parent,
            EpisodeProperties {
                author,
                creation_time,
                description,
                last_modify_time,
                path,
                scenes,
                start_with,
                version,
            },
        )))
    }
}

impl EpisodeState {
    pub fn back(&mut self) -> RunnerResult<()> {
        // BACK
        todo!()
    }

    pub fn get_current_scene(&self) -> RunnerResult<()> {
        // GETCURRENTSCENE
        todo!()
    }

    pub fn get_latest_scene(&self) -> RunnerResult<()> {
        // GETLATESTSCENE
        todo!()
    }

    pub fn go_to(&mut self, episode: &Episode, scene_name: &str) -> RunnerResult<()> {
        // GOTO
        episode.parent.parent.runner.change_scene(scene_name)
    }

    pub fn next(&mut self) -> RunnerResult<()> {
        // NEXT
        todo!()
    }

    pub fn prev(&mut self) -> RunnerResult<()> {
        // PREV
        todo!()
    }

    pub fn restart(&mut self) -> RunnerResult<()> {
        // RESTART
        todo!()
    }
}
