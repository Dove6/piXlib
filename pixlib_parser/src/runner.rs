use std::{
    collections::{HashMap, VecDeque},
    ops::{Add, Div, Mul, Rem, Sub},
    path::Path,
    sync::Arc,
};

use crate::{
    ast::{
        Expression, IgnorableProgram, IgnorableStatement, Invocation, Operation, Program, Statement,
    },
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

#[derive(Debug, Default, Clone)]
pub struct CnvRunner {
    scripts: HashMap<Arc<Path>, CnvScript>,
}

pub struct RunnerContext {
    pub self_object: String,
    pub current_object: String,
}

impl CnvRunner {
    // pub fn step(&mut self, )

    pub fn load_script(
        &mut self,
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
        while let Some(Ok((_pos, dec, _))) = dec_parser.next_if(|result| result.is_ok()) {
            match dec {
                CnvDeclaration::ObjectInitialization(name) => {
                    objects.push(CnvObjectBuilder::new(name.clone(), objects.len()));
                    name_to_object
                        .insert(name, objects.len() - 1)
                        .warn_if_some();
                }
                CnvDeclaration::PropertyAssignment {
                    parent,
                    property,
                    property_key: _property_key,
                    value,
                } => {
                    let Some(obj) = name_to_object
                        .get(&parent)
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
        let objects: Vec<Arc<CnvObject>> = objects
            .into_iter()
            .filter_map(|builder| match builder.build() {
                Ok(built_object) => Some(Arc::new(built_object)),
                Err(e) => {
                    issue_manager.emit_issue(e);
                    None
                }
            })
            .collect();

        self.scripts.insert(
            Arc::clone(&path),
            CnvScript {
                source_kind,
                path,
                parent_path,
                objects,
            },
        ); // TODO: err if present
        Ok(())
    }

    pub fn get_script(&self, path: &Path) -> Option<&CnvScript> {
        self.scripts.get(path)
    }

    pub fn get_script_mut(&mut self, path: &Path) -> Option<&mut CnvScript> {
        self.scripts.get_mut(path)
    }

    pub fn get_root_script(&self) -> Option<&CnvScript> {
        self.scripts
            .values()
            .find(|s| s.source_kind == ScriptSource::Root)
    }

    pub fn find_scripts(
        &self,
        predicate: impl Fn(&CnvScript) -> bool,
        buffer: &mut Vec<Arc<Path>>,
    ) {
        buffer.clear();
        for (path, script) in self.scripts.iter() {
            if predicate(script) {
                buffer.push(Arc::clone(path));
            }
        }
    }

    pub fn unload_all_scripts(&mut self) {
        self.scripts.clear();
    }

    pub fn unload_script(&mut self, path: &Path) {
        let mut traversing_queue: VecDeque<&Path> = VecDeque::new();
        traversing_queue.push_back(path);
        let mut unloading_queue: Vec<Arc<Path>> = Vec::new();
        while let Some(current) = traversing_queue.pop_front() {
            unloading_queue.push(current.into());
            for (key, value) in self.scripts.iter() {
                if value
                    .parent_path
                    .as_ref()
                    .is_some_and(|p| current == p.as_ref())
                {
                    traversing_queue.push_back(key.as_ref());
                }
            }
        }
        while let Some(current) = unloading_queue.pop() {
            self.scripts.remove(&current);
        }
    }

    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        // println!("Getting object: {:?}", name);
        for script in self.scripts.values() {
            for object in script.objects.iter() {
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
        for script in self.scripts.values() {
            for object in script.objects.iter() {
                if predicate(object) {
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
        let Some(script) = self.get_script_mut(&script_name) else {
            return Err(BehaviorRunningError::ScriptNotFound);
        };
        let Some(init_beh_obj) = script.get_object(name) else {
            return Err(BehaviorRunningError::ObjectNotFound);
        };
        if init_beh_obj.content.read().unwrap().get_type_id() != "BEHAVIOUR" {
            return Err(BehaviorRunningError::InvalidType);
        };
        let mut context = RunnerContext {
            self_object: init_beh_obj.name.clone(),
            current_object: init_beh_obj.name.clone(),
        };
        init_beh_obj.call_method(
            CallableIdentifier::Method("RUN"),
            &Vec::new(),
            self,
            &mut context,
        );
        Ok(None)
    }
}

pub enum BehaviorRunningError {
    ScriptNotFound,
    ObjectNotFound,
    InvalidType,
}

#[derive(Debug, Clone)]
pub struct CnvScript {
    pub source_kind: ScriptSource,
    pub path: Arc<Path>,
    pub parent_path: Option<Arc<Path>>,
    pub objects: Vec<Arc<CnvObject>>,
}

impl CnvScript {
    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        for object in self.objects.iter() {
            if object.name == name {
                return Some(Arc::clone(object));
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
        for object in self.objects.iter() {
            if predicate(object) {
                buffer.push(Arc::clone(object));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptSource {
    Root,
    Application,
    Episode,
    Scene,
    CnvLoader,
}

#[allow(dead_code)]
struct CnvApplication {
    application_name: String,
}

#[allow(dead_code)]
impl CnvApplication {
    // @BOOL
    // @BREAK
    // @CONTINUE
    // @CONV
    // @CREATE
    // @DOUBLE
    // @FOR
    // @GETAPPLICATIONNAME
    // @GETCURRENTSCENE
    // @IF
    // @INT
    // @LOOP
    // @MSGBOX
    // @ONEBREAK
    // @RETURN
    // @RUNONTIMER
    // @STRING
    // @VALUE
    // @WHILE

    fn get_application_name(&self) -> &str {
        &self.application_name
    }
}

#[derive(Debug)]
pub enum CnvValue {
    Integer(i32),
    Double(f64),
    Boolean(bool),
    String(String),
    Reference(Arc<CnvObject>),
}

impl CnvValue {
    pub fn to_integer(&self) -> i32 {
        match self {
            CnvValue::Integer(i) => *i,
            CnvValue::Double(d) => *d as i32,
            CnvValue::Boolean(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            CnvValue::String(_) => 0,
            CnvValue::Reference(_) => todo!(),
        }
    }

    pub fn to_double(&self) -> f64 {
        match self {
            CnvValue::Integer(i) => (*i).into(),
            CnvValue::Double(d) => *d,
            CnvValue::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            CnvValue::String(_) => 0.0,
            CnvValue::Reference(_) => todo!(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            CnvValue::Integer(i) => *i != 0,  // TODO: check
            CnvValue::Double(d) => *d != 0.0, // TODO: check
            CnvValue::Boolean(b) => *b,
            CnvValue::String(s) => !s.is_empty(), // TODO: check
            CnvValue::Reference(_) => todo!(),
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            CnvValue::Integer(i) => i.to_string(),
            CnvValue::Double(d) => d.to_string(), // TODO: check
            CnvValue::Boolean(b) => b.to_string(), //TODO: check
            CnvValue::String(s) => s.clone(),
            CnvValue::Reference(r) => r.name.clone(), // TODO: not always
        }
    }
}

impl Add for &CnvValue {
    type Output = CnvValue;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i + rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d + rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b || rhs.to_boolean()),
            CnvValue::String(s) => CnvValue::String(s.clone() + rhs.to_string().as_ref()),
            CnvValue::Reference(_) => todo!(),
        }
    }
}

impl Mul for &CnvValue {
    type Output = CnvValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i * rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d * rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b && rhs.to_boolean()),
            CnvValue::String(s) => CnvValue::String(s.clone()),
            CnvValue::Reference(_) => todo!(),
        }
    }
}

impl Sub for &CnvValue {
    type Output = CnvValue;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i - rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d - rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b && !rhs.to_boolean()),
            CnvValue::String(s) => CnvValue::String(s.clone()),
            CnvValue::Reference(_) => todo!(),
        }
    }
}

impl Div for &CnvValue {
    type Output = CnvValue;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i / rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d / rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b),
            CnvValue::String(s) => CnvValue::String(s.clone()),
            CnvValue::Reference(_) => todo!(),
        }
    }
}

impl Rem for &CnvValue {
    type Output = CnvValue;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i % rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d % rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b),
            CnvValue::String(s) => CnvValue::String(s.clone()),
            CnvValue::Reference(_) => todo!(),
        }
    }
}

pub trait CnvExpression {
    fn calculate(&self, runner: &mut CnvRunner, context: &mut RunnerContext) -> Option<CnvValue>;
}

pub trait CnvStatement {
    fn run(&self, runner: &mut CnvRunner, context: &mut RunnerContext);
}

impl CnvExpression for Invocation {
    fn calculate(&self, runner: &mut CnvRunner, context: &mut RunnerContext) -> Option<CnvValue> {
        // println!("Invocation::calculate: {:?}", self);
        if self.parent.is_none() {
            None // TODO: match &self.name
        } else {
            let parent = self
                .parent
                .as_ref()
                .unwrap()
                .calculate(runner, context)
                .expect("Invalid invocation parent");
            let arguments: Vec<_> = self
                .arguments
                .iter()
                .map(|e| e.calculate(runner, context))
                .collect();
            let arguments: Vec<_> = arguments.into_iter().map(|e| e.unwrap()).collect();
            // println!("Calling method: {:?} of: {:?}", self.name, self.parent);
            match parent {
                CnvValue::Reference(obj) => obj.call_method(
                    CallableIdentifier::Method(&self.name),
                    &arguments,
                    runner,
                    context,
                ),
                _ => panic!(
                    "Expected invocation parent to be an object, got {:?}",
                    parent
                ),
            }
        }
    }
}

impl CnvExpression for Expression {
    fn calculate(&self, runner: &mut CnvRunner, context: &mut RunnerContext) -> Option<CnvValue> {
        // println!("Expression::calculate: {:?}", self);
        match self {
            Expression::LiteralBool(b) => Some(CnvValue::Boolean(*b)),
            Expression::Identifier(name) => runner
                .get_object(name[..].trim_matches('\"'))
                .map(CnvValue::Reference)
                .or_else(|| Some(CnvValue::String(name.trim_matches('\"').to_owned()))),
            Expression::SelfReference => runner
                .get_object(&context.self_object)
                .map(CnvValue::Reference), // error
            Expression::Parameter(_name) => None, // access function scope and retrieve arguments
            Expression::NameResolution(expression) => {
                let _name = &expression.calculate(runner, context);
                let name = String::new(); // TODO: stringify
                runner.get_object(&name[..]).map(CnvValue::Reference) // error
            }
            Expression::FieldAccess(_expression, _field) => todo!(),
            Expression::Operation(expression, operations) => {
                let mut result = expression
                    .calculate(runner, context)
                    .expect("Expected non-void argument in operation");
                for (operation, argument) in operations {
                    let argument = argument
                        .calculate(runner, context)
                        .expect("Expected non-void argument in operation");
                    result = match operation {
                        Operation::Addition => &result + &argument,
                        Operation::Multiplication => &result * &argument,
                        Operation::Subtraction => &result - &argument,
                        Operation::Division => &result / &argument,
                        Operation::Remainder => &result % &argument,
                    }
                }
                Some(result)
            }
            Expression::Block(_block) => todo!(), // create a temporary function
        }
    }
}

impl CnvStatement for IgnorableProgram {
    fn run(&self, runner: &mut CnvRunner, context: &mut RunnerContext) {
        // println!("IgnorableProgram::run: {:?}", self);
        if self.ignored {
            return;
        }
        self.value.run(runner, context);
    }
}

impl CnvStatement for Program {
    fn run(&self, runner: &mut CnvRunner, context: &mut RunnerContext) {
        // println!("Program::run: {:?}", self);
        match self {
            Program::Identifier(identifier) => {
                let obj = runner
                    .get_object(identifier)
                    .unwrap_or_else(|| panic!("Expected existing object named {}", &identifier));
                obj.call_method(
                    CallableIdentifier::Method("RUN"),
                    &Vec::new(),
                    runner,
                    context,
                );
            }
            Program::Block(ignorable_statements) => {
                for ignorable_statement in ignorable_statements {
                    ignorable_statement.run(runner, context);
                }
            }
        }
    }
}

impl CnvStatement for IgnorableStatement {
    fn run(&self, runner: &mut CnvRunner, context: &mut RunnerContext) {
        // println!("IgnorableStatement::run: {:?}", self);
        if self.ignored {
            return;
        }
        self.value.run(runner, context);
    }
}

impl CnvStatement for Statement {
    fn run(&self, runner: &mut CnvRunner, context: &mut RunnerContext) {
        // println!("Statement::run: {:?}", self);
        match self {
            Statement::Invocation(invocation) => {
                invocation.calculate(runner, context);
            }
            Statement::ExpressionStatement(expression) => {
                expression.calculate(runner, context);
            }
        }
    }
}
