mod events;
mod expression;
mod object_container;
mod script;
mod script_container;
mod statement;
mod value;

pub use expression::CnvExpression;
pub use script::{CnvScript, ScriptSource};
use script_container::ScriptContainer;
pub use statement::CnvStatement;
pub use value::CnvValue;

use std::{cell::RefCell, collections::HashMap, path::Path, sync::Arc};

use events::{IncomingEvents, OutgoingEvents};

use crate::{
    classes::{CallableIdentifier, CnvObject, CnvObjectBuilder, ObjectBuilderError},
    common::{Issue, IssueHandler, IssueManager},
    declarative_parser::{
        CnvDeclaration, DeclarativeParser, ParserFatal, ParserInput, ParserIssue,
    },
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

#[derive(Debug, Clone)]
pub struct CnvRunner {
    pub scripts: RefCell<ScriptContainer>,
    pub events_in: IncomingEvents,
    pub events_out: OutgoingEvents,
    pub current_scene: Option<Arc<CnvObject>>,
    pub filesystem: Arc<RefCell<dyn FileSystem>>,
}

#[derive(Debug)]
pub struct RunnerContext<'a> {
    pub runner: &'a mut CnvRunner,
    pub self_object: String,
    pub current_object: String,
}

pub trait FileSystem: std::fmt::Debug + Send + Sync {
    fn read_file(&self, filename: &str) -> std::io::Result<Vec<u8>>;
    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()>;
}

#[derive(Debug)]
pub struct DummyFileSystem;

impl FileSystem for DummyFileSystem {
    fn read_file(&self, _: &str) -> std::io::Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn write_file(&mut self, _: &str, _: &[u8]) -> std::io::Result<()> {
        Ok(())
    }
}

impl CnvRunner {
    pub fn new(filesystem: Arc<RefCell<dyn FileSystem>>) -> Self {
        Self {
            scripts: RefCell::new(ScriptContainer::default()),
            filesystem,
            current_scene: None,
            events_in: IncomingEvents::default(),
            events_out: OutgoingEvents::default(),
        }
    }

    pub fn load_script(
        self: &Arc<Self>,
        path: Arc<Path>,
        contents: impl Iterator<Item = ParserInput>,
        parent_path: Option<Arc<Path>>,
        source_kind: ScriptSource,
        issue_manager: &mut IssueManager<ObjectBuilderError>,
    ) -> Result<(), ParserFatal> {
        let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
        parser_issue_manager.set_handler(Box::new(IssuePrinter));
        let mut dec_parser =
            DeclarativeParser::new(contents, Default::default(), parser_issue_manager).peekable();
        let mut objects: Vec<CnvObjectBuilder> = Vec::new();
        let mut name_to_object: HashMap<String, usize> = HashMap::new();
        let script = Arc::new(CnvScript::new(
            Arc::clone(self),
            Arc::clone(&path),
            parent_path,
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
        container.push_script(script); // TODO: err if present
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

    pub fn unload_script(&self, path: &Path) {
        self.scripts.borrow_mut().remove_script(path);
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
        &mut self,
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
            runner: self,
            self_object: init_beh_obj.name.clone(),
            current_object: init_beh_obj.name.clone(),
        };
        init_beh_obj.call_method(CallableIdentifier::Method("RUN"), &Vec::new(), &mut context);
        Ok(None)
    }

    pub fn change_scene(&mut self, _scene_name: &str) -> Result<(), ()> {
        todo!()
    }

    pub fn get_current_scene(&self) -> Option<Arc<CnvObject>> {
        self.current_scene.as_ref().map(Arc::clone)
    }
}

pub enum BehaviorRunningError {
    ScriptNotFound,
    ObjectNotFound,
    InvalidType,
}
