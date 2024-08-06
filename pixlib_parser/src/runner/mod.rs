mod events;
mod expression;
mod filesystem;
mod object_container;
mod script;
mod script_container;
mod statement;
mod value;

pub use events::{KeyboardEvent, MouseEvent, ScriptEvent, TimerEvent, KeyboardKey};
pub use expression::CnvExpression;
pub use filesystem::FileSystem;
pub use filesystem::GamePaths;
pub use script::{CnvScript, ScriptSource};
use script_container::ScriptContainer;
pub use statement::CnvStatement;
use thiserror::Error;
pub use value::CnvValue;

use std::{cell::RefCell, collections::HashMap, path::Path, sync::Arc};

use events::{IncomingEvents, OutgoingEvents};

use crate::classes::Animation;
use crate::classes::Scene;
use crate::common::IssueKind;
use crate::scanner::parse_cnv;
use crate::{
    classes::{CallableIdentifier, CnvObject, CnvObjectBuilder, ObjectBuilderError},
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
    TooManyArguments { expected_max: usize, actual: usize },
    TooFewArguments { expected_min: usize, actual: usize },
    MissingLeftOperand,
    MissingRightOperand,
    MissingOperator,
    ObjectNotFound { name: String },
    NoDataLoaded,
    SequenceNameNotFound { name: String },
    IoError { source: std::io::Error },
}

pub type RunnerResult<T> = std::result::Result<T, RunnerError>;

#[derive(Clone)]
pub struct CnvRunner {
    pub scripts: RefCell<ScriptContainer>,
    pub events_in: IncomingEvents,
    pub events_out: OutgoingEvents,
    pub current_scene: RefCell<Option<Arc<CnvObject>>>,
    pub filesystem: Arc<RefCell<dyn FileSystem>>,
    pub game_paths: Arc<GamePaths>,
    pub issue_manager: Arc<RefCell<IssueManager<RunnerIssue>>>,
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
            .field("current_scene", &self.current_scene)
            .field("filesystem", &self.filesystem)
            .field("game_paths", &self.game_paths)
            .field("issue_manager", &self.issue_manager)
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

#[derive(Debug)]
pub struct RunnerContext {
    pub runner: Arc<CnvRunner>,
    pub self_object: String,
    pub current_object: String,
}

impl CnvRunner {
    pub fn new(
        filesystem: Arc<RefCell<dyn FileSystem>>,
        game_paths: Arc<GamePaths>,
        issue_manager: IssueManager<RunnerIssue>,
    ) -> Self {
        Self {
            scripts: RefCell::new(ScriptContainer::default()),
            filesystem,
            current_scene: RefCell::new(None),
            events_in: IncomingEvents::default(),
            events_out: OutgoingEvents::default(),
            game_paths,
            issue_manager: Arc::new(RefCell::new(issue_manager)),
        }
    }

    pub fn step(self: &Arc<CnvRunner>) -> Result<(), RunnerError> {
        let mut timer_events = self.events_in.timer.borrow_mut();
        while let Some(evt) = timer_events.pop_front() {
            match evt {
                TimerEvent::Elapsed { seconds } => {
                    let mut buffer = Vec::new();
                    self.find_objects(|o| o.content.borrow().as_ref().unwrap().get_type_id() == "ANIMO", &mut buffer);
                    for animation_object in buffer {
                        let mut guard = animation_object.content.borrow_mut();
                        let animation = guard.as_mut().unwrap().as_any_mut().downcast_mut::<Animation>().unwrap();
                        let mut context = RunnerContext { current_object: animation_object.name.clone(), self_object: animation_object.name.clone(), runner: Arc::clone(self) };
                        animation.tick(&mut context, seconds)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_script(
        self: &Arc<Self>,
        path: Arc<Path>,
        contents: impl Iterator<Item = declarative_parser::ParserInput>,
        parent_object: Option<Arc<CnvObject>>,
        source_kind: ScriptSource,
    ) -> Result<(), ParserFatal> {
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
            Arc::clone(&path),
            parent_object.clone(),
            source_kind,
        ));
        while let Some(Ok((_pos, dec, _))) = dec_parser.next_if(|result| result.is_ok()) {
            match dec {
                CnvDeclaration::ObjectInitialization(name) => {
                    objects.push(CnvObjectBuilder::new(
                        Arc::clone(&script),
                        Arc::clone(&path),
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
            return Err(err);
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
                            None
                        }
                    }),
            )
            .map_err(|_e| ParserFatal::Other)?;

        let mut container = self.scripts.borrow_mut();
        container
            .push_script(script)
            .map_err(|_| ParserFatal::Other)?; // TODO: err if present
        if let Some(parent_object) = parent_object.as_ref() {
            if parent_object
                .content
                .borrow()
                .as_ref()
                .unwrap()
                .get_type_id()
                == "SCENE"
            {
                self.current_scene
                    .borrow_mut()
                    .replace(Arc::clone(&parent_object));
            }
        }
        self.events_out
            .script
            .borrow_mut()
            .push_back(ScriptEvent::ScriptLoaded {
                path: Arc::clone(&path),
            });
        Ok(())
    }

    pub fn get_script(&self, path: &Path) -> Option<Arc<CnvScript>> {
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

    pub fn unload_script(&self, path: &Path) -> Result<(), ()> {
        self.scripts.borrow_mut().remove_script(path)
    }

    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        // println!("Getting object: {:?}", name);
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
        for script in self.scripts.borrow().iter() {
            for object in script.objects.borrow().iter() {
                if predicate(&object) {
                    buffer.push(Arc::clone(object));
                }
            }
        }
    }

    pub fn run_behavior(
        self: &Arc<CnvRunner>,
        script_name: Arc<Path>,
        name: &str,
    ) -> Result<Option<CnvValue>, BehaviorRunningError> {
        let Some(script) = self.get_script(&script_name) else {
            return Err(BehaviorRunningError::ScriptNotFound);
        };
        let Some(init_beh_obj) = script.get_object(name) else {
            return Err(BehaviorRunningError::ObjectNotFound);
        };
        if init_beh_obj
            .content
            .borrow()
            .as_ref()
            .unwrap()
            .get_type_id()
            != "BEHAVIOUR"
        {
            return Err(BehaviorRunningError::InvalidType);
        };
        let mut context = RunnerContext {
            runner: Arc::clone(self),
            self_object: init_beh_obj.name.clone(),
            current_object: init_beh_obj.name.clone(),
        };
        init_beh_obj
            .call_method(CallableIdentifier::Method("RUN"), &Vec::new(), &mut context)
            .map_err(|e| BehaviorRunningError::RunnerError(e))?;
        Ok(None)
    }

    pub fn change_scene(self: &Arc<Self>, scene_name: &str) -> Result<(), ()> {
        self.scripts.borrow_mut().remove_scene_script()?;
        let Some(scene_object) = self.get_object(scene_name) else {
            return Err(());
        };
        let scene_guard = scene_object.content.borrow();
        let scene = scene_guard
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<Scene>()
            .unwrap();
        let scene_name = scene_object.name.clone();
        let scene_path = scene.get_script_path();
        let (contents, path) = self
            .filesystem
            .borrow()
            .read_scene_file(
                self.game_paths.clone(),
                Some(&scene_path.unwrap()),
                &scene_name,
                Some("CNV"),
            )
            .unwrap();
        let contents = parse_cnv(&contents);
        self.load_script(path, contents.as_parser_input(), None, ScriptSource::Scene)
            .map_err(|_| ())
    }

    pub fn get_current_scene(&self) -> Option<Arc<CnvObject>> {
        self.current_scene.borrow().as_ref().map(Arc::clone)
    }
}

pub enum BehaviorRunningError {
    ScriptNotFound,
    ObjectNotFound,
    InvalidType,
    RunnerError(RunnerError),
}
