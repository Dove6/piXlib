use std::{any::Any, cell::RefCell};

use super::super::{
    content::EventHandler,
    parsers::{discard_if_empty, parse_comma_separated, parse_datetime},
};

use crate::parser::ast::ParsedScript;

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ApplicationProperties {
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
struct ApplicationState {
    // deduced from methods
    pub has_music_enabled: bool,
    pub language_code: String,
    pub is_being_dragged: bool,
}

#[derive(Debug, Clone)]
pub struct ApplicationEventHandlers {}

impl EventHandler for ApplicationEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    parent: Arc<CnvObject>,

    state: RefCell<ApplicationState>,
    event_handlers: ApplicationEventHandlers,

    author: String,
    bloomoo_version: String,
    creation_time: Option<DateTime<Utc>>,
    description: String,
    episodes: Vec<EpisodeName>,
    last_modify_time: Option<DateTime<Utc>>,
    path: Option<String>,
    start_with: Option<EpisodeName>,
    version: String,
}

impl Application {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: ApplicationProperties) -> Self {
        let mut app = Self {
            parent,
            state: RefCell::new(ApplicationState {
                has_music_enabled: true,
                language_code: "040E".to_owned(),
                is_being_dragged: false,
            }),
            event_handlers: ApplicationEventHandlers {},
            author: props.author.unwrap_or_default(),
            bloomoo_version: props.bloomoo_version.unwrap_or_default(),
            creation_time: props.creation_time,
            description: props.description.unwrap_or_default(),
            episodes: props.episodes.unwrap_or_default(),
            last_modify_time: props.last_modify_time,
            path: props.path,
            start_with: props.start_with,
            version: props.version.unwrap_or_default(),
        };
        if app.start_with.is_none() && !app.episodes.is_empty() {
            app.start_with = Some(app.episodes[0].clone());
        }
        app
    }

    // custom

    pub fn get_episode_list(&self) -> Vec<String> {
        self.episodes.clone()
    }

    pub fn get_starting_episode(&self) -> Option<String> {
        self.start_with.clone().or(self.episodes.first().cloned())
    }

    pub fn get_script_path(&self) -> Option<String> {
        self.path.clone()
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("DISABLEMUSIC") => self
                .state
                .borrow_mut()
                .disable_music()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("ENABLEMUSIC") => self
                .state
                .borrow_mut()
                .enable_music()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("EXISTSENV") => {
                self.state.borrow().exists_env().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("EXIT") => self
                .state
                .borrow_mut()
                .exit(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETLANGUAGE") => {
                self.state.borrow().get_language().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETPLAYER") => {
                self.state.borrow().get_player().map(CnvValue::String)
            }
            CallableIdentifier::Method("GOTO") => {
                self.state.borrow_mut().goto().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("PRINT") => {
                self.state.borrow_mut().print().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RELOAD") => {
                self.state.borrow_mut().reload().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RESTART") => {
                self.state.borrow_mut().restart().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RUN") => {
                self.state.borrow_mut().run().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RUNENV") => {
                self.state.borrow_mut().run_env().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETLANGUAGE") => self
                .state
                .borrow_mut()
                .set_language()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("STARTDRAGGINGWINDOW") => self
                .state
                .borrow_mut()
                .start_dragging_window()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("STOPDRAGGINGWINDOW") => self
                .state
                .borrow_mut()
                .stop_dragging_window()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("STOREBINARY") => self
                .state
                .borrow_mut()
                .store_binary()
                .map(|_| CnvValue::Null),
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
        Ok(CnvContent::Application(Self::from_initial_properties(
            parent,
            ApplicationProperties {
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
        )))
    }
}

impl ApplicationState {
    pub fn disable_music(&mut self) -> anyhow::Result<()> {
        // DISABLEMUSIC
        todo!()
    }

    pub fn enable_music(&mut self) -> anyhow::Result<()> {
        // ENABLEMUSIC
        todo!()
    }

    pub fn exists_env(&self) -> anyhow::Result<bool> {
        // EXISTSENV
        todo!()
    }

    pub fn exit(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // EXIT
        context
            .runner
            .events_out
            .app
            .borrow_mut()
            .use_and_drop_mut(|events| events.push_back(ApplicationEvent::ApplicationExited));
        Ok(())
    }

    pub fn get_language(&self) -> anyhow::Result<String> {
        // GETLANGUAGE
        todo!()
    }

    pub fn get_player(&self) -> anyhow::Result<String> {
        // GETPLAYER
        todo!()
    }

    pub fn goto(&mut self) -> anyhow::Result<()> {
        // GOTO
        todo!()
    }

    pub fn print(&mut self) -> anyhow::Result<()> {
        // PRINT
        todo!()
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        // RELOAD
        todo!()
    }

    pub fn restart(&mut self) -> anyhow::Result<()> {
        // RESTART
        todo!()
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        // RUN
        todo!()
    }

    pub fn run_env(&mut self) -> anyhow::Result<()> {
        // RUNENV
        todo!()
    }

    pub fn set_language(&mut self) -> anyhow::Result<()> {
        // SETLANGUAGE
        todo!()
    }

    pub fn start_dragging_window(&mut self) -> anyhow::Result<()> {
        // STARTDRAGGINGWINDOW
        todo!()
    }

    pub fn stop_dragging_window(&mut self) -> anyhow::Result<()> {
        // STOPDRAGGINGWINDOW
        todo!()
    }

    pub fn store_binary(&mut self) -> anyhow::Result<()> {
        // STOREBINARY
        todo!()
    }
}
