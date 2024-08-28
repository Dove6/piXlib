use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::parsers::{discard_if_empty, parse_comma_separated, parse_datetime};

use crate::parser::ast::ParsedScript;

use super::super::common::*;
use super::super::*;
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
pub struct EpisodeState {
    previous_scene_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EpisodeEventHandlers {}

impl EventHandler for EpisodeEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

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

    // custom

    pub fn get_script_path(&self) -> Option<String> {
        self.path.clone()
    }

    pub fn get_scene_list(&self) -> Vec<String> {
        self.scenes.clone()
    }

    pub fn get_starting_scene(&self) -> Option<String> {
        self.start_with.clone().or(self.scenes.first().cloned())
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
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        // log::trace!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("BACK") => self
                .state
                .borrow_mut()
                .back(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETCURRENTSCENE") => self
                .state
                .borrow()
                .get_current_scene()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETLATESTSCENE") => self
                .state
                .borrow()
                .get_latest_scene()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GOTO") => self
                .state
                .borrow_mut()
                .go_to(context, &arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("NEXT") => {
                self.state.borrow_mut().next().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("PREV") => {
                self.state.borrow_mut().prev().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RESTART") => {
                self.state.borrow_mut().restart().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context).map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
            }
            ident => Err(RunnerError::InvalidCallable {
                object_name: self.parent.name.clone(),
                callable: ident.to_owned(),
            }
            .into()),
        }
    }

    fn new_content(
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
    pub fn back(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // BACK
        let current_scene_name = context.runner.get_current_scene().map(|s| s.name.clone());
        let goto_name = self
            .previous_scene_name
            .take()
            .or(current_scene_name.clone())
            .unwrap();
        self.previous_scene_name = current_scene_name;
        context.runner.change_scene(&goto_name)
    }

    pub fn get_current_scene(&self) -> anyhow::Result<()> {
        // GETCURRENTSCENE
        todo!()
    }

    pub fn get_latest_scene(&self) -> anyhow::Result<()> {
        // GETLATESTSCENE
        todo!()
    }

    pub fn go_to(&mut self, context: RunnerContext, scene_name: &str) -> anyhow::Result<()> {
        // GOTO
        self.previous_scene_name = context.runner.get_current_scene().map(|s| s.name.clone());
        context.runner.change_scene(scene_name)
    }

    pub fn next(&mut self) -> anyhow::Result<()> {
        // NEXT
        todo!()
    }

    pub fn prev(&mut self) -> anyhow::Result<()> {
        // PREV
        todo!()
    }

    pub fn restart(&mut self) -> anyhow::Result<()> {
        // RESTART
        todo!()
    }
}
