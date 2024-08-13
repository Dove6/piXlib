mod events;
mod expression;
mod filesystem;
mod object_container;
mod path;
mod script;
mod script_container;
mod statement;
#[cfg(test)]
mod tests;
mod value;

pub use events::{
    ApplicationEvent, FileEvent, GraphicsEvent, InternalEvent, KeyboardEvent, KeyboardKey,
    MouseEvent, ObjectEvent, ScriptEvent, SoundEvent, TimerEvent,
};
pub use expression::CnvExpression;
pub use filesystem::FileSystem;
pub use filesystem::GamePaths;
use itertools::Itertools;
use object_container::ObjectContainer;
pub use path::ScenePath;
pub use script::{CnvScript, ScriptSource};
use script_container::ScriptContainer;
pub use statement::CnvStatement;
use thiserror::Error;
pub use value::CnvValue;

use std::collections::VecDeque;
use std::fmt::Display;
use std::{cell::RefCell, collections::HashMap, sync::Arc};

use events::{IncomingEvents, OutgoingEvents};

use crate::classes::Animation;
use crate::classes::CnvContent;
use crate::classes::Scene;
use crate::common::DroppableRefMut;
use crate::common::IssueKind;
use crate::scanner::parse_cnv;
use crate::{
    classes::{CnvObject, CnvObjectBuilder, ObjectBuilderError},
    common::{Issue, IssueHandler, IssueManager},
    declarative_parser::{self, CnvDeclaration, DeclarativeParser, ParserFatal, ParserIssue},
};

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        eprintln!("{:?}", issue);
    }
}

trait SomeWarnable {
    fn warn_if_some(&self);
}

impl<T> SomeWarnable for Option<T>
where
    T: std::fmt::Debug,
{
    fn warn_if_some(&self) {
        if self.is_some() {
            eprintln!("Unexpected value: {:?}", self.as_ref().unwrap());
        }
    }
}

#[derive(Debug)]
pub enum RunnerError {
    TooManyArguments {
        expected_max: usize,
        actual: usize,
    },
    TooFewArguments {
        expected_min: usize,
        actual: usize,
    },
    MissingLeftOperand {
        object_name: String,
    },
    MissingRightOperand {
        object_name: String,
    },
    MissingOperator {
        object_name: String,
    },
    ObjectNotFound {
        name: String,
    },
    NoDataLoaded,
    SequenceNameNotFound {
        object_name: String,
        sequence_name: String,
    },

    MissingFilenameToLoad,

    ScriptNotFound {
        path: String,
    },
    RootScriptAlreadyLoaded,
    ApplicationScriptAlreadyLoaded,
    EpisodeScriptAlreadyLoaded,
    SceneScriptAlreadyLoaded,

    ParserError(ParserFatal),

    IoError {
        source: std::io::Error,
    },
    Other,
}

pub type RunnerResult<T> = std::result::Result<T, RunnerError>;

#[derive(Clone)]
pub struct CnvRunner {
    pub scripts: RefCell<ScriptContainer>,
    pub events_in: IncomingEvents,
    pub events_out: OutgoingEvents,
    pub internal_events: RefCell<VecDeque<InternalEvent>>,
    pub filesystem: Arc<RefCell<dyn FileSystem>>,
    pub game_paths: Arc<GamePaths>,
    pub issue_manager: Arc<RefCell<IssueManager<RunnerIssue>>>,
    pub global_objects: RefCell<ObjectContainer>,
}

impl core::fmt::Debug for CnvRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CnvRunner")
            .field(
                "scripts",
                &self.scripts.borrow().iter().map(|o| {
                    (
                        o.parent_object.as_ref().map(|p| p.name.clone()),
                        o.path.clone(),
                    )
                }),
            )
            .field("events_in", &self.events_in)
            .field("events_out", &self.events_out)
            .field("internal_events", &self.internal_events)
            .field("filesystem", &self.filesystem)
            .field("game_paths", &self.game_paths)
            .field("issue_manager", &self.issue_manager)
            .field("global_objects", &self.global_objects)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum RunnerIssue {}

impl Issue for RunnerIssue {
    fn kind(&self) -> IssueKind {
        IssueKind::Error
    }
}

#[derive(Debug, Clone)]
pub struct RunnerContext {
    pub runner: Arc<CnvRunner>,
    pub self_object: Arc<CnvObject>,
    pub current_object: Arc<CnvObject>,
    pub arguments: Vec<CnvValue>,
}

impl Display for RunnerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RunnerContext {{ self: {}, current: {}, arguments: [{}] }}",
            self.self_object.name,
            self.current_object.name,
            self.arguments.iter().map(|v| format!("{}", v)).join(", ")
        )
    }
}

impl RunnerContext {
    pub fn new(
        runner: &Arc<CnvRunner>,
        self_object: &Arc<CnvObject>,
        current_object: &Arc<CnvObject>,
        arguments: &[CnvValue],
    ) -> Self {
        Self {
            runner: runner.clone(),
            self_object: self_object.clone(),
            current_object: current_object.clone(),
            arguments: arguments.to_owned(),
        }
    }

    pub fn new_minimal(runner: &Arc<CnvRunner>, current_object: &Arc<CnvObject>) -> Self {
        Self {
            runner: runner.clone(),
            self_object: current_object.clone(),
            current_object: current_object.clone(),
            arguments: Vec::new(),
        }
    }

    pub fn with_current_object(self, current_object: Arc<CnvObject>) -> Self {
        Self {
            current_object,
            ..self
        }
    }

    pub fn with_arguments(self, arguments: Vec<CnvValue>) -> Self {
        Self { arguments, ..self }
    }
}

impl CnvRunner {
    pub fn new(
        filesystem: Arc<RefCell<dyn FileSystem>>,
        game_paths: Arc<GamePaths>,
        issue_manager: IssueManager<RunnerIssue>,
    ) -> Arc<Self> {
        let runner = Arc::new(Self {
            scripts: RefCell::new(ScriptContainer::default()),
            filesystem,
            events_in: IncomingEvents::default(),
            events_out: OutgoingEvents::default(),
            internal_events: RefCell::new(VecDeque::new()),
            game_paths,
            issue_manager: Arc::new(RefCell::new(issue_manager)),
            global_objects: RefCell::new(ObjectContainer::default()),
        });
        let global_script = Arc::new(CnvScript::new(
            Arc::clone(&runner),
            ScenePath {
                dir_path: ".".into(),
                file_path: "__GLOBAL__".into(),
            },
            None,
            ScriptSource::Root,
        ));
        runner
            .global_objects
            .borrow_mut()
            .use_and_drop_mut(|objects| {
                objects
                    .push_object({
                        let mut builder = CnvObjectBuilder::new(
                            Arc::clone(&global_script),
                            "RANDOM".to_owned(),
                            0,
                        );
                        builder.add_property("TYPE".into(), "RAND".to_owned());
                        builder.build().unwrap()
                    })
                    .unwrap()
            });
        runner
    }

    pub fn step(self: &Arc<CnvRunner>) -> RunnerResult<()> {
        let mut timer_events = self.events_in.timer.borrow_mut();
        while let Some(evt) = timer_events.pop_front() {
            match evt {
                TimerEvent::Elapsed { seconds } => {
                    let mut buffer = Vec::new();
                    self.find_objects(
                        |o| matches!(&*o.content.borrow(), CnvContent::Animation(_)),
                        &mut buffer,
                    );
                    for animation_object in buffer {
                        let guard = animation_object.content.borrow();
                        let animation: Option<&Animation> = (&*guard).into();
                        let animation = animation.unwrap();
                        animation.step(seconds)?;
                    }
                }
            }
        }
        let mut to_init = Vec::new();
        self.find_objects(|o| !*o.initialized.borrow(), &mut to_init);
        for object in to_init {
            object.init(None)?;
        }
        while let Some(evt) = self
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| events.pop_front())
        {
            evt.object
                .call_method((&evt.callable).into(), &evt.arguments, None)?;
        }
        Ok(())
    }

    pub fn load_script(
        self: &Arc<Self>,
        path: ScenePath,
        contents: impl Iterator<Item = declarative_parser::ParserInput>,
        parent_object: Option<Arc<CnvObject>>,
        source_kind: ScriptSource,
    ) -> RunnerResult<()> {
        let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
        parser_issue_manager.set_handler(Box::new(IssuePrinter));
        let mut issue_manager: IssueManager<ObjectBuilderError> = Default::default();
        issue_manager.set_handler(Box::new(IssuePrinter));
        let mut dec_parser =
            DeclarativeParser::new(contents, Default::default(), parser_issue_manager).peekable();
        let mut objects: Vec<CnvObjectBuilder> = Vec::new();
        let mut name_to_object: HashMap<String, usize> = HashMap::new();
        let script = Arc::new(CnvScript::new(
            Arc::clone(self),
            path.clone(),
            parent_object.clone(),
            source_kind,
        ));
        while let Some(Ok((_pos, dec, _))) = dec_parser.next_if(|result| result.is_ok()) {
            match dec {
                CnvDeclaration::ObjectInitialization(name) => {
                    objects.push(CnvObjectBuilder::new(
                        Arc::clone(&script),
                        name.trim().to_owned(),
                        objects.len(),
                    ));
                    name_to_object
                        .insert(name.trim().to_owned(), objects.len() - 1)
                        .warn_if_some();
                }
                CnvDeclaration::PropertyAssignment {
                    parent,
                    property,
                    property_key: _property_key,
                    value,
                } => {
                    let Some(obj) = name_to_object
                        .get(parent.trim())
                        .and_then(|i| objects.get_mut(*i))
                    else {
                        panic!(
                            "Expected {} element to be in dict, the element list is: {:?}",
                            &parent, &objects
                        );
                    };
                    obj.add_property(property, value);
                }
            }
        }
        if let Some(Err(err)) = dec_parser.next_if(|result| result.is_err()) {
            return Err(RunnerError::ParserError(err));
        }
        script
            .objects
            .borrow_mut()
            .push_objects(
                objects
                    .into_iter()
                    .filter_map(|builder| match builder.build() {
                        Ok(built_object) => Some(built_object),
                        Err(e) => {
                            issue_manager.emit_issue(e);
                            panic!();
                        }
                    }),
            )?;

        let mut container = self.scripts.borrow_mut();
        container.push_script(script)?; // TODO: err if present
        self.events_out
            .script
            .borrow_mut()
            .push_back(ScriptEvent::ScriptLoaded { path: path.clone() });
        Ok(())
    }

    pub fn get_script(&self, path: &ScenePath) -> Option<Arc<CnvScript>> {
        self.scripts.borrow().get_script(path)
    }

    pub fn get_root_script(&self) -> Option<Arc<CnvScript>> {
        self.scripts.borrow().get_root_script()
    }

    pub fn find_scripts(
        &self,
        predicate: impl Fn(&CnvScript) -> bool,
        buffer: &mut Vec<Arc<CnvScript>>,
    ) {
        buffer.clear();
        for script in self.scripts.borrow().iter() {
            if predicate(script.as_ref()) {
                buffer.push(Arc::clone(&script));
            }
        }
    }

    pub fn unload_all_scripts(&self) {
        self.scripts.borrow_mut().remove_all_scripts();
    }

    pub fn unload_script(&self, path: &ScenePath) -> RunnerResult<()> {
        self.scripts.borrow_mut().remove_script(path)
    }

    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        // println!("Getting object: {:?}", name);
        for object in self.global_objects.borrow().iter() {
            if object.name == name {
                return Some(Arc::clone(object));
            }
        }
        for script in self.scripts.borrow().iter() {
            for object in script.objects.borrow().iter() {
                if object.name == name {
                    return Some(Arc::clone(object));
                }
            }
        }
        None
    }

    pub fn find_object(&self, predicate: impl Fn(&CnvObject) -> bool) -> Option<Arc<CnvObject>> {
        for object in self.global_objects.borrow().iter() {
            if predicate(object) {
                return Some(Arc::clone(object));
            }
        }
        for script in self.scripts.borrow().iter() {
            for object in script.objects.borrow().iter() {
                if predicate(&object) {
                    return Some(Arc::clone(object));
                }
            }
        }
        None
    }

    pub fn find_objects(
        &self,
        predicate: impl Fn(&CnvObject) -> bool,
        buffer: &mut Vec<Arc<CnvObject>>,
    ) {
        buffer.clear();
        for object in self.global_objects.borrow().iter() {
            if predicate(object) {
                buffer.push(Arc::clone(object));
            }
        }
        for script in self.scripts.borrow().iter() {
            for object in script.objects.borrow().iter() {
                if predicate(&object) {
                    buffer.push(Arc::clone(object));
                }
            }
        }
    }

    pub fn change_scene(self: &Arc<Self>, scene_name: &str) -> RunnerResult<()> {
        self.scripts.borrow_mut().remove_scene_script()?;
        let Some(scene_object) = self.get_object(scene_name) else {
            return Err(RunnerError::ObjectNotFound {
                name: scene_name.to_owned(),
            });
        };
        let scene_guard = scene_object.content.borrow();
        let scene: Option<&Scene> = (&*scene_guard).into();
        let scene = scene.unwrap();
        let scene_name = scene_object.name.clone();
        let scene_path = scene.get_script_path().unwrap();
        let contents = (*self.filesystem)
            .borrow_mut()
            .read_scene_file(
                self.game_paths.clone(),
                &ScenePath::new(&scene_path, &(scene_name.clone() + ".cnv")),
            )
            .unwrap();
        let contents = parse_cnv(&contents);
        self.load_script(
            ScenePath::new(&scene_path, &scene_name),
            contents.as_parser_input(),
            Some(Arc::clone(&scene_object)),
            ScriptSource::Scene,
        )
    }

    pub fn get_current_scene(&self) -> Option<Arc<CnvObject>> {
        self.scripts
            .borrow()
            .get_scene_script()
            .and_then(|s| s.parent_object.as_ref().cloned())
    }
}

pub enum BehaviorRunningError {
    ScriptNotFound,
    ObjectNotFound,
    InvalidType,
    RunnerError(RunnerError),
}
