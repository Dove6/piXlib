mod animation;
mod application;
mod array;
mod behavior;
mod bool;
mod button;
mod canvasobserver;
mod cnvloader;
mod complexcondition;
mod condition;
mod dbl;
mod episode;
mod expression;
mod font;
mod group;
mod image;
mod int;
mod keyboard;
mod mouse;
mod multiarray;
mod music;
mod random;
mod scene;
mod sequence;
mod sound;
mod str;
mod structure;
mod system;
mod text;
mod timer;

pub use animation::Animation;
pub use application::Application;
pub use array::Array;
pub use behavior::Behavior;
pub use bool::Bool;
pub use button::Button;
pub use canvasobserver::CanvasObserver;
pub use cnvloader::CnvLoader;
pub use complexcondition::ComplexCondition;
pub use condition::Condition;
pub use dbl::Dbl;
pub use episode::Episode;
pub use expression::Expression;
pub use font::Font;
pub use group::Group;
pub use image::Image;
pub use int::Int;
pub use keyboard::Keyboard;
pub use lalrpop_util::ParseError;
pub use mouse::Mouse;
pub use multiarray::MultiArray;
pub use music::Music;
pub use random::Random;
pub use scene::Scene;
pub use sequence::Sequence;
pub use sound::Sound;
pub use str::Str;
pub use structure::Struct;
pub use system::System;
pub use text::Text;
pub use timer::Timer;

use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    path::{Path, PathBuf},
    sync::Arc,
    vec::IntoIter,
};
use thiserror::Error;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    ast::{IgnorableProgram, ParserFatal, ParserIssue},
    common::{Issue, IssueHandler, IssueManager, Position},
    lexer::{CnvLexer, CnvToken},
    parser::CodeParser,
    runner::{CnvScript, CnvStatement, CnvValue, FileSystem, RunnerContext},
    scanner::CnvScanner,
};

pub type EpisodeName = String;
pub type SceneName = String;
pub type ConditionName = String;
pub type ImageName = String;
pub type SoundName = String;
pub type VariableName = String;
pub type TypeName = String;
pub type FontName = String;

#[derive(Debug, Clone)]
pub struct CnvObjectBuilder {
    parent: Arc<RefCell<CnvScript>>,
    path: Arc<Path>,
    name: String,
    index: usize,
    properties: HashMap<String, String>,
}

impl CnvObjectBuilder {
    pub fn new(
        parent: Arc<RefCell<CnvScript>>,
        path: Arc<Path>,
        name: String,
        index: usize,
    ) -> Self {
        Self {
            parent,
            path,
            name,
            index,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, property: String, value: String) {
        self.properties.insert(property, value); // TODO: report duplicates
    }

    pub fn build(self) -> Result<Arc<CnvObject>, ObjectBuilderError> {
        let mut properties = self.properties;
        let Some(type_name) = properties.remove("TYPE").and_then(discard_if_empty) else {
            return Err(ObjectBuilderError::new(
                self.name,
                ObjectBuildErrorKind::MissingType,
            )); // TODO: readable errors
        };
        let object = Arc::new(CnvObject {
            parent: self.parent,
            name: self.name.clone(),
            index: self.index,
            content: RefCell::new(None),
        });
        let content =
            CnvTypeFactory::create(Arc::clone(&object), type_name, properties).map_err(|e| {
                ObjectBuilderError::new(self.name, ObjectBuildErrorKind::ParsingError(e))
            })?;
        object.content.replace(Some(content));
        Ok(object)
    }
}

#[derive(Debug, Error)]
#[error("Error building object {name}: {source}")]
pub struct ObjectBuilderError {
    pub name: String,
    pub path: Arc<Path>,
    pub source: Box<ObjectBuildErrorKind>,
}

impl ObjectBuilderError {
    pub fn new(name: String, source: ObjectBuildErrorKind) -> Self {
        Self {
            name,
            path: PathBuf::from(".").into(),
            source: Box::new(source),
        }
    }
}

impl Issue for ObjectBuilderError {
    fn kind(&self) -> crate::common::IssueKind {
        match *self.source {
            ObjectBuildErrorKind::ParsingError(TypeParsingError::InvalidProgram(
                ProgramParsingError(ParseError::User { .. }),
            )) => crate::common::IssueKind::Fatal,
            _ => crate::common::IssueKind::Fatal,
        }
    }
}

#[derive(Debug, Error)]
pub enum ObjectBuildErrorKind {
    #[error("Missing type property")]
    MissingType,
    #[error("Parsing error: {0}")]
    ParsingError(TypeParsingError),
}

#[derive(Debug)]
pub struct CnvObject {
    pub parent: Arc<RefCell<CnvScript>>,
    pub name: String,
    pub index: usize,
    pub content: RefCell<Option<Box<dyn CnvType>>>,
}

#[derive(Debug)]
pub enum CallableIdentifier<'a> {
    Method(&'a str),
    Event(&'a str),
}

impl CnvObject {
    pub fn call_method(
        &self,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        println!("Calling method: {:?} of: {:?}", identifier, self.name);
        self.content
            .borrow_mut()
            .as_mut()
            .unwrap()
            .call_method(identifier, arguments, context)
    }

    pub fn get_property(&self, name: &str) -> Option<PropertyValue> {
        println!("Getting property: {:?} of: {:?}", name, self.name);
        self.content.borrow().as_ref().unwrap().get_property(name)
    }
}

#[derive(Debug)]
pub enum MemberInfo<'a> {
    Property(PropertyInfo<'a>),
    Callable(CallableInfo<'a>),
}

#[derive(Debug)]
pub struct PropertyInfo<'a> {
    name: &'a str,
    r#type: PropertyValue,
}

#[derive(Debug)]
pub struct CallableInfo<'a> {
    identifier: CallableIdentifier<'a>,
    parameters: &'a [PropertyInfo<'a>],
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

pub trait CnvType: std::fmt::Debug {
    fn get_type_id(&self) -> &'static str;
    fn has_event(&self, name: &str) -> bool;
    fn has_property(&self, name: &str) -> bool;
    fn has_method(&self, name: &str) -> bool;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_property(&self, name: &str) -> Option<PropertyValue>;
    fn call_method(
        &mut self,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>>;

    fn new(
        parent: Arc<CnvObject>,
        properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError>
    where
        Self: Sized;
}

impl dyn CnvType {}

#[derive(Debug)]
pub struct DummyCnvType {}

impl CnvType for DummyCnvType {
    fn get_type_id(&self) -> &'static str {
        "DUMMY"
    }

    fn has_event(&self, name: &str) -> bool {
        false
    }

    fn has_property(&self, name: &str) -> bool {
        false
    }

    fn has_method(&self, name: &str) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        None
    }

    fn call_method(
        &mut self,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        Ok(None)
    }

    fn new(
        parent: Arc<CnvObject>,
        properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError>
    where
        Self: Sized,
    {
        Ok(Self {})
    }
}

pub struct CnvTypeFactory;

impl CnvTypeFactory {
    pub fn create(
        parent: Arc<CnvObject>,
        type_name: String,
        properties: HashMap<String, String>,
    ) -> Result<Box<dyn CnvType>, TypeParsingError> {
        match type_name.as_ref() {
            "ANIMO" => Animation::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "APPLICATION" => {
                Application::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "ARRAY" => Array::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "BEHAVIOUR" => {
                Behavior::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "BOOL" => Bool::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "BUTTON" => Button::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "CANVAS_OBSERVER" => {
                CanvasObserver::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CANVASOBSERVER" => {
                CanvasObserver::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CNVLOADER" => {
                CnvLoader::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CONDITION" => {
                Condition::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "COMPLEXCONDITION" => {
                ComplexCondition::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "DOUBLE" => Dbl::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "EPISODE" => Episode::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "EXPRESSION" => {
                Expression::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "FONT" => Font::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "GROUP" => Group::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "IMAGE" => Image::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "INTEGER" => Int::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "KEYBOARD" => {
                Keyboard::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "MOUSE" => Mouse::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "MULTIARRAY" => {
                MultiArray::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "MUSIC" => Music::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "RANDOM" => Random::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SCENE" => Scene::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SEQUENCE" => {
                Sequence::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "SOUND" => Sound::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "STRING" => Str::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "STRUCT" => Struct::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SYSTEM" => System::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "TEXT" => Text::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "TIMER" => Timer::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            _ => Err(TypeParsingError::UnknownType(type_name)),
        }
    }
}

#[derive(Debug, Error)]
pub enum TypeParsingError {
    #[error("Unknown type: {0}")]
    UnknownType(String),
    #[error("Invalid bool literal: {0}")]
    InvalidBoolLiteral(String),
    #[error("Invalid integer literal: {0}")]
    InvalidIntegerLiteral(ParseIntError),
    #[error("Invalid floating-point literal: {0}")]
    InvalidFloatingLiteral(ParseFloatError),
    #[error("Invalid rect literal: {0}")]
    InvalidRectLiteral(String),
    #[error("Invalid condition operator: {0}")]
    InvalidConditionOperator(String),
    #[error("Invalid complex condition operator: {0}")]
    InvalidComplexConditionOperator(String),
    #[error("Invalid expression operator: {0}")]
    InvalidExpressionOperator(String),
    #[error("Invalid program: {0}")]
    InvalidProgram(ProgramParsingError),
}

#[derive(Debug, Error)]
pub struct ProgramParsingError(pub ParseError<Position, CnvToken, ParserFatal>);

impl Display for ProgramParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ParseError::InvalidToken { location } => write!(f, "Invalid token at {}", location),
            ParseError::UnrecognizedEof { location, expected } => {
                write!(f, "Unexpected EOF at {}, expected {:?}", location, expected)
            }
            ParseError::UnrecognizedToken { token, expected } => {
                write!(f, "Unexpected token {:?}, expected {:?}", token, expected)
            }
            ParseError::ExtraToken { token } => write!(f, "Extra token {:?}", token),
            ParseError::User { error } => write!(f, "{}", error),
        }
    }
}

impl From<ParseError<Position, CnvToken, ParserFatal>> for TypeParsingError {
    fn from(value: ParseError<Position, CnvToken, ParserFatal>) -> Self {
        TypeParsingError::InvalidProgram(ProgramParsingError(value))
    }
}

fn parse_bool(s: String) -> Result<bool, TypeParsingError> {
    match s.as_ref() {
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        _ => Err(TypeParsingError::InvalidBoolLiteral(s)),
    }
}

fn parse_i32(s: String) -> Result<i32, TypeParsingError> {
    s.parse().map_err(TypeParsingError::InvalidIntegerLiteral)
}

fn parse_f64(s: String) -> Result<f64, TypeParsingError> {
    s.parse().map_err(TypeParsingError::InvalidFloatingLiteral)
}

fn parse_datetime(_s: String) -> Result<DateTime<Utc>, TypeParsingError> {
    Ok(DateTime::default()) // TODO: parse date
}

fn parse_comma_separated(s: String) -> Result<Vec<String>, TypeParsingError> {
    Ok(s.split(',').map(|s| s.trim().to_owned()).collect())
}

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        eprintln!("{:?}", issue);
    }
}

fn parse_program(s: String) -> Result<Arc<IgnorableProgram>, TypeParsingError> {
    let scanner = CnvScanner::<IntoIter<_>>::new(s.chars().map(Ok).collect::<Vec<_>>().into_iter());
    let lexer = CnvLexer::new(scanner, Default::default(), Default::default());
    let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
    parser_issue_manager.set_handler(Box::new(IssuePrinter));
    Ok(Arc::new(CodeParser::new().parse(
        &Default::default(),
        &mut parser_issue_manager,
        lexer,
    )?))
}

fn parse_rect(s: String) -> Result<Rect, TypeParsingError> {
    if s.contains(',') {
        s.split(',')
            .map(|s| s.parse().unwrap())
            .collect_tuple()
            .map(Rect::Literal)
            .ok_or(TypeParsingError::InvalidRectLiteral(s))
    } else {
        Ok(Rect::Reference(s))
    }
}

fn discard_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[derive(Debug)]
pub enum PropertyValue {
    Boolean(bool),
    Integer(i32),
    Double(f64),
    String(String),
    List(Vec<String>),
    Rect(Rect),
    Time(DateTime<Utc>),
    Code(Arc<IgnorableProgram>),
}

impl From<bool> for PropertyValue {
    fn from(value: bool) -> Self {
        PropertyValue::Boolean(value)
    }
}

impl From<i32> for PropertyValue {
    fn from(value: i32) -> Self {
        PropertyValue::Integer(value)
    }
}

impl From<f64> for PropertyValue {
    fn from(value: f64) -> Self {
        PropertyValue::Double(value)
    }
}

impl From<String> for PropertyValue {
    fn from(value: String) -> Self {
        PropertyValue::String(value)
    }
}

impl From<Vec<String>> for PropertyValue {
    fn from(value: Vec<String>) -> Self {
        PropertyValue::List(value)
    }
}

impl From<Rect> for PropertyValue {
    fn from(value: Rect) -> Self {
        PropertyValue::Rect(value)
    }
}

impl From<DateTime<Utc>> for PropertyValue {
    fn from(value: DateTime<Utc>) -> Self {
        PropertyValue::Time(value)
    }
}

impl From<Arc<IgnorableProgram>> for PropertyValue {
    fn from(value: Arc<IgnorableProgram>) -> Self {
        PropertyValue::Code(value)
    }
}

#[derive(Debug, Clone)]
pub enum GraphicsEvents {
    Play(String),
    Pause,
    Stop(bool),
    Finished(String),
    Show,
    Hide,
}

#[derive(Debug, Clone)]
pub enum Rect {
    Literal((i32, i32, i32, i32)),
    Reference(String),
}

impl Default for Rect {
    fn default() -> Self {
        Self::Literal(Default::default())
    }
}

#[derive(Debug, Clone)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

impl ConditionOperator {
    pub fn parse(s: String) -> Result<Self, TypeParsingError> {
        match s.as_ref() {
            "EQUAL" => Ok(Self::Equal),
            "NOTEQUAL" => Ok(Self::NotEqual),
            "LESS" => Ok(Self::Less),
            "GREATER" => Ok(Self::Greater),
            "LESSEQUAL" => Ok(Self::LessEqual),
            "GREATEREQUAL" => Ok(Self::GreaterEqual),
            _ => Err(TypeParsingError::InvalidConditionOperator(s)), // TODO: error
        }
    }
}

#[derive(Debug, Clone)]
pub enum ComplexConditionOperator {
    And,
    Or,
}

impl ComplexConditionOperator {
    pub fn parse(s: String) -> Result<Self, TypeParsingError> {
        match s.as_ref() {
            "AND" => Ok(Self::And),
            "OR" => Ok(Self::Or),
            _ => Err(TypeParsingError::InvalidComplexConditionOperator(s)), // TODO: error
        }
    }
}

#[derive(Debug, Clone)]
pub enum EpisodeEvents {
    GoTo(String),
}

#[derive(Default, Debug, Clone)]
pub enum ExpressionOperator {
    #[default]
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl ExpressionOperator {
    pub fn parse(s: String) -> Result<Self, TypeParsingError> {
        match s.as_ref() {
            "ADD" => Ok(Self::Add),
            "SUB" => Ok(Self::Sub),
            "MUL" => Ok(Self::Mul),
            "DIV" => Ok(Self::Div),
            "MOD" => Ok(Self::Mod),
            _ => Err(TypeParsingError::InvalidExpressionOperator(s)), // TODO: something better
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FontDef {
    family: String,
    style: String,
    size: usize,
}

lazy_static! {
    static ref STRUCT_FIELDS_REGEX: Regex = Regex::new(r"^(\w+)<(\w+)>$").unwrap();
}
