use std::{
    collections::{HashMap, VecDeque},
    io::Read,
    ops::{Add, Div, Mul, Rem, Sub},
    path::Path,
    sync::Arc,
};

use crate::{
    ast::{
        Expression, IgnorableProgram, IgnorableStatement, Invocation, Operation, Program, Statement,
    },
    classes::{CnvObject, CnvObjectBuilder},
    common::{Issue, IssueHandler, IssueManager},
    declarative_parser::{CnvDeclaration, DeclarativeParser, ParserIssue},
    scanner::{CnvDecoder, CnvHeader, CnvScanner, CodepageDecoder, CP1250_LUT},
};

#[derive(Debug, Default, Clone)]
pub struct CnvRunner {
    scripts: HashMap<Arc<Path>, CnvScript>,
}

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        eprintln!("{:?}", issue);
    }
}

trait SomePanicable {
    fn and_panic(&self);
}

impl<T> SomePanicable for Option<T> {
    fn and_panic(&self) {
        if self.is_some() {
            panic!();
        }
    }
}

#[allow(dead_code)]
impl CnvRunner {
    pub fn load_script(
        &mut self,
        path: &Path,
        parent_path: Option<Arc<Path>>,
        source_kind: ScriptSource,
    ) -> std::io::Result<Arc<Path>> {
        let full_path: Arc<Path> = path
            .canonicalize()
            .map(|p| p.into())
            .unwrap_or(path.to_owned().into());

        let input = std::fs::File::open(&full_path)?;
        let mut input = input.bytes().peekable();
        let mut first_line = Vec::<u8>::new();
        while let Some(res) =
            input.next_if(|res| res.as_ref().is_ok_and(|c| !matches!(c, b'\r' | b'\n')))
        {
            first_line.push(res.unwrap())
        }
        while let Some(res) =
            input.next_if(|res| res.as_ref().is_ok_and(|c| matches!(c, b'\r' | b'\n')))
        {
            first_line.push(res.unwrap())
        }
        let input: Box<dyn Iterator<Item = std::io::Result<u8>>> =
            match CnvHeader::try_new(&first_line) {
                Ok(Some(CnvHeader {
                    cipher_class: _,
                    step_count,
                })) => Box::new(CnvDecoder::new(input, step_count)),
                Ok(None) => Box::new(first_line.into_iter().map(Ok).chain(input)),
                Err(err) => panic!("{}", err),
            };
        let decoder = CodepageDecoder::new(&CP1250_LUT, input);
        let scanner = CnvScanner::new(decoder);
        let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
        parser_issue_manager.set_handler(Box::new(IssuePrinter));
        let mut dec_parser =
            DeclarativeParser::new(scanner, Default::default(), parser_issue_manager).peekable();
        let mut objects: HashMap<String, CnvObjectBuilder> = HashMap::new();
        let mut counter: usize = 0;
        while let Some(Ok((_pos, dec, _))) = dec_parser.next_if(|result| result.is_ok()) {
            match dec {
                CnvDeclaration::ObjectInitialization(name) => {
                    objects
                        .insert(name.clone(), CnvObjectBuilder::new(name, counter))
                        .and_panic();
                    counter += 1;
                }
                CnvDeclaration::PropertyAssignment {
                    parent,
                    property,
                    property_key: _property_key,
                    value,
                } => {
                    let Some(obj) = objects.get_mut(&parent) else {
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
            return Err(std::io::Error::other(err));
        }
        let objects: HashMap<String, CnvObject> = objects
            .into_iter()
            .map(|(name, builder)| (name, builder.build().unwrap()))
            .collect();

        self.scripts.insert(
            Arc::clone(&full_path),
            CnvScript {
                source_kind,
                path: Arc::clone(&full_path),
                parent_path,
                objects,
            },
        );
        Ok(full_path)
    }

    pub fn get_script(&self, path: &Path) -> Option<&CnvScript> {
        self.scripts.get(path)
    }

    pub fn get_script_mut(&mut self, path: &Path) -> Option<&mut CnvScript> {
        self.scripts.get_mut(path)
    }

    pub fn unload_script(&mut self, path: &Path) -> Result<(), &'static str> {
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
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CnvScript {
    pub source_kind: ScriptSource,
    pub path: Arc<Path>,
    pub parent_path: Option<Arc<Path>>,
    pub objects: HashMap<String, CnvObject>,
}

#[derive(Debug, Clone, Copy)]
pub enum ScriptSource {
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
    //     @BOOL
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

pub enum CnvValue {
    Integer(i32),
    Double(f64),
    Boolean(bool),
    String(String),
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
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            CnvValue::Integer(i) => *i != 0,  // TODO: check
            CnvValue::Double(d) => *d != 0.0, // TODO: check
            CnvValue::Boolean(b) => *b,
            CnvValue::String(s) => !s.is_empty(), // TODO: check
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            CnvValue::Integer(i) => i.to_string(),
            CnvValue::Double(d) => d.to_string(), // TODO: check
            CnvValue::Boolean(b) => b.to_string(), //TODO: check
            CnvValue::String(s) => s.clone(),
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
        }
    }
}

pub type CnvObjects = HashMap<String, Box<CnvObject>>;

pub trait CnvExpression {
    fn calculate(&self, objects: &mut CnvObjects) -> Option<CnvValue>;
}

pub trait CnvStatement {
    fn run(&self, objects: &mut CnvObjects);
}

impl CnvExpression for Invocation {
    fn calculate(&self, objects: &mut CnvObjects) -> Option<CnvValue> {
        if self.parent.is_none() {
            None // TODO: match &self.name
        } else {
            let _parent = self.parent.as_ref().unwrap().calculate(objects);
            match objects.get_mut("") {
                // TODO: stringify parent
                Some(obj) => obj.call_method(&self.name),
                None => None, // error
            }
        }
    }
}

impl CnvExpression for Expression {
    fn calculate(&self, objects: &mut CnvObjects) -> Option<CnvValue> {
        match self {
            Expression::LiteralBool(b) => Some(CnvValue::Boolean(*b)),
            Expression::Identifier(name) => objects.get_mut(&name[..]).and_then(|x| x.get_value()), // error
            Expression::Parameter(_name) => None, // access function scope and retrieve arguments
            Expression::NameResolution(expression) => {
                let _name = &expression.calculate(objects);
                let name = String::new(); // TODO: stringify
                objects.get_mut(&name[..]).and_then(|x| x.get_value()) // error
            }
            Expression::FieldAccess(_expression, _field) => todo!(),
            Expression::Operation(expression, operations) => {
                let mut result = expression
                    .calculate(objects)
                    .expect("Expected non-void argument in operation");
                for (operation, argument) in operations {
                    let argument = argument
                        .calculate(objects)
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
    fn run(&self, objects: &mut CnvObjects) {
        if self.ignored {
            return;
        }
        self.value.run(objects);
    }
}

impl CnvStatement for Program {
    fn run(&self, objects: &mut CnvObjects) {
        match self {
            Program::Identifier(identifier) => {
                let _obj = objects
                    .get(identifier)
                    .unwrap_or_else(|| panic!("Expected existing object named {}", &identifier));
                todo!(); // run object
            }
            Program::Block(ignorable_statements) => {
                for ignorable_statement in ignorable_statements {
                    ignorable_statement.run(objects);
                }
            }
        }
    }
}

impl CnvStatement for IgnorableStatement {
    fn run(&self, objects: &mut CnvObjects) {
        if self.ignored {
            return;
        }
        self.value.run(objects);
    }
}

impl CnvStatement for Statement {
    fn run(&self, objects: &mut CnvObjects) {
        match self {
            Statement::Invocation(invocation) => {
                invocation.calculate(objects);
            }
            Statement::ExpressionStatement(expression) => {
                expression.calculate(objects);
            }
        }
    }
}
