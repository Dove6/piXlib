use lalrpop_util::ParseError;
use std::{
    any::Any,
    collections::HashMap,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
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
    runner::{CnvRunner, CnvStatement, CnvValue, RunnerContext},
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
    name: String,
    index: usize,
    properties: HashMap<String, String>,
}

impl CnvObjectBuilder {
    pub fn new(name: String, index: usize) -> Self {
        Self {
            name,
            index,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, property: String, value: String) {
        self.properties.insert(property, value); // TODO: report duplicates
    }

    pub fn build(self) -> Result<CnvObject, ObjectBuilderError> {
        let mut properties = self.properties;
        let Some(type_name) = properties.remove("TYPE").and_then(discard_if_empty) else {
            return Err(ObjectBuilderError::new(
                self.name,
                ObjectBuildErrorKind::MissingType,
            )); // TODO: readable errors
        };
        let content = CnvTypeFactory::create(type_name, properties).map_err(|e| {
            ObjectBuilderError::new(self.name.clone(), ObjectBuildErrorKind::ParsingError(e))
        })?;
        Ok(CnvObject {
            name: self.name,
            index: self.index,
            content: RwLock::new(content),
        })
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
    pub name: String,
    pub index: usize,
    pub content: RwLock<Box<dyn CnvType>>,
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
        script_runner: &mut CnvRunner,
        context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        // println!("Calling method: {:?} of: {:?}", identifier, self);
        self.content
            .write()
            .unwrap()
            .call_method(identifier, arguments, script_runner, context)
    }

    pub fn get_property(&self, name: &str) -> Option<PropertyValue> {
        self.content.read().unwrap().get_property(name)
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

pub trait CnvType: Send + Sync + std::fmt::Debug {
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
        script_runner: &mut CnvRunner,
        context: &mut RunnerContext,
    ) -> Option<CnvValue>;

    fn new(properties: HashMap<String, String>) -> Result<Self, TypeParsingError>
    where
        Self: Sized;
}

impl dyn CnvType {}

pub struct CnvTypeFactory;

impl CnvTypeFactory {
    pub fn create(
        type_name: String,
        properties: HashMap<String, String>,
    ) -> Result<Box<dyn CnvType>, TypeParsingError> {
        match type_name.as_ref() {
            "ANIMO" => Animation::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "APPLICATION" => Application::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "ARRAY" => Array::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "BEHAVIOUR" => Behavior::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "BOOL" => Bool::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "BUTTON" => Button::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "CANVAS_OBSERVER" => {
                CanvasObserver::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CANVASOBSERVER" => {
                CanvasObserver::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CNVLOADER" => CnvLoader::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "CONDITION" => Condition::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "COMPLEXCONDITION" => {
                ComplexCondition::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "DOUBLE" => Dbl::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "EPISODE" => Episode::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "EXPRESSION" => Expression::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "FONT" => Font::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "GROUP" => Group::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "IMAGE" => Image::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "INTEGER" => Int::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "KEYBOARD" => Keyboard::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "MOUSE" => Mouse::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "MULTIARRAY" => MultiArray::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "MUSIC" => Music::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "RANDOM" => Random::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SCENE" => Scene::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SEQUENCE" => Sequence::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SOUND" => Sound::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "STRING" => Str::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "STRUCT" => Struct::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SYSTEM" => System::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "TEXT" => Text::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "TIMER" => Timer::new(properties).map(|o| Box::new(o) as Box<dyn CnvType>),
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
    Stop(bool),
    Finished(String),
    Show,
    Hide,
}

#[derive(Debug, Clone)]
pub struct Animation {
    // ANIMO
    pub as_button: Option<bool>,                    // ASBUTTON
    pub filename: Option<String>,                   // FILENAME
    pub flush_after_played: Option<bool>,           // FLUSHAFTERPLAYED
    pub fps: Option<i32>,                           // FPS
    pub monitor_collision: Option<bool>,            // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>,      // MONITORCOLLISIONALPHA
    pub preload: Option<bool>,                      // PRELOAD
    pub priority: Option<i32>,                      // PRIORITY
    pub release: Option<bool>,                      // RELEASE
    pub to_canvas: Option<bool>,                    // TOCANVAS
    pub visible: Option<bool>,                      // VISIBLE
    pub on_init: Option<Arc<IgnorableProgram>>,     // ONINIT signal
    pub on_finished: Option<Arc<IgnorableProgram>>, // ONFINISHED signal

    pub events: Vec<GraphicsEvents>,
}

impl CnvType for Animation {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "ANIMO"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONCLICK"
                | "ONCOLLISION"
                | "ONCOLLISIONFINISHED"
                | "ONDONE"
                | "ONFINISHED"
                | "ONFIRSTFRAME"
                | "ONFOCUSOFF"
                | "ONFOCUSON"
                | "ONFRAMECHANGED"
                | "ONINIT"
                | "ONPAUSED"
                | "ONRELEASE"
                | "ONRESUMED"
                | "ONSIGNAL"
                | "ONSTARTED"
        )
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        script_runner: &mut CnvRunner,
        context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("PLAY") => {
                self.events.push(GraphicsEvents::Play(arguments[0].to_string()));
                None
            }
            CallableIdentifier::Method("STOP") => {
                self.events.push(GraphicsEvents::Stop(arguments[0].to_boolean()));
                None
            }
            CallableIdentifier::Method("HIDE") => {
                self.events.push(GraphicsEvents::Hide);
                None
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.on_init.as_ref() {
                    v.run(script_runner, context)
                }
                None
            }
            CallableIdentifier::Event("ONFINISHED") => {
                if let Some(v) = self.on_finished.as_ref() {
                    v.run(script_runner, context)
                }
                None
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "FILENAME" => self.filename.clone().map(|v| v.into()),
            "PRIORITY" => self.priority.map(|v| v.into()),
            "ONINIT" => self.on_init.as_ref().map(|v| Arc::clone(v).into()),
            "ONFINISHED" => self.on_finished.as_ref().map(|v| Arc::clone(v).into()),
            _ => todo!(),
        }
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let as_button = properties
            .remove("ASBUTTON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let fps = properties
            .remove("FPS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let release = properties
            .remove("RELEASE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_finished = properties
            .remove("ONFINISHED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self {
            as_button,
            filename,
            flush_after_played,
            fps,
            monitor_collision,
            monitor_collision_alpha,
            preload,
            priority,
            release,
            to_canvas,
            visible,
            on_init,
            on_finished,
            events: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Application {
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
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "PATH" => self.path.clone().map(|v| v.into()),
            "EPISODES" => self.episodes.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
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
        Ok(Self {
            author,
            bloomoo_version,
            creation_time,
            description,
            episodes,
            last_modify_time,
            path,
            start_with,
            version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    // ARRAY
    send_on_change: Option<bool>, // SENDONCHANGE
}

impl CnvType for Array {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "ARRAY"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let send_on_change = properties
            .remove("SENDONCHANGE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        // TODO: too many properties
        Ok(Self { send_on_change })
    }
}

#[derive(Debug, Clone)]
pub struct Behavior {
    // BEHAVIOUR
    pub code: Option<Arc<IgnorableProgram>>, // CODE
    pub condition: Option<ConditionName>,    // CONDITION
}

impl CnvType for Behavior {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BEHAVIOUR"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(name, "ONDONE" | "ONINIT" | "ONSIGNAL")
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let code = properties
            .remove("CODE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let condition = properties.remove("CONDITION").and_then(discard_if_empty);
        Ok(Self { code, condition })
    }
}

#[derive(Debug, Clone)]
pub struct Bool {
    // BOOL
    default: Option<bool>,   // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<bool>,     // VALUE
}

impl CnvType for Bool {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BOOL"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties
            .remove("VALUE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
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
pub struct Button {
    // BUTTON
    accent: Option<bool>,            // ACCENT
    drag: Option<bool>,              // DRAG
    draggable: Option<bool>,         // DRAGGABLE
    enable: Option<bool>,            // ENABLE
    gfx_on_click: Option<ImageName>, // GFXONCLICK
    gfx_on_move: Option<ImageName>,  // GFXONMOVE
    gfx_standard: Option<ImageName>, // GFXSTANDARD
    priority: Option<i32>,           // PRIORITY
    rect: Option<Rect>,              // RECT
    snd_on_click: Option<SoundName>, // SNDONCLICK
    snd_on_move: Option<SoundName>,  // SNDONMOVE
    snd_standard: Option<SoundName>, // SNDSTANDARD
}

impl CnvType for Button {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BUTTON"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let accent = properties
            .remove("ACCENT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let drag = properties
            .remove("DRAG")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let draggable = properties
            .remove("DRAGGABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let enable = properties
            .remove("ENABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let gfx_on_click = properties.remove("GFXONCLICK").and_then(discard_if_empty);
        let gfx_on_move = properties.remove("GFXONMOVE").and_then(discard_if_empty);
        let gfx_standard = properties.remove("GFXSTANDARD").and_then(discard_if_empty);
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let rect = properties
            .remove("RECT")
            .and_then(discard_if_empty)
            .map(parse_rect)
            .transpose()?;
        let snd_on_click = properties.remove("SNDONCLICK").and_then(discard_if_empty);
        let snd_on_move = properties.remove("SNDONMOVE").and_then(discard_if_empty);
        let snd_standard = properties.remove("SNDSTANDARD").and_then(discard_if_empty);
        Ok(Self {
            accent,
            drag,
            draggable,
            enable,
            gfx_on_click,
            gfx_on_move,
            gfx_standard,
            priority,
            rect,
            snd_on_click,
            snd_on_move,
            snd_standard,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CanvasObserver {
    // CANVAS_OBSERVER
}

impl CnvType for CanvasObserver {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "CANVASOBSERVER"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(_properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        Ok(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct CnvLoader {
    // CNVLOADER
    cnv_loader: Option<String>, // CNVLOADER
}

impl CnvType for CnvLoader {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "CNVLOADER"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let cnv_loader = properties.remove("CNVLOADER").and_then(discard_if_empty);
        Ok(Self { cnv_loader })
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
pub struct Condition {
    // CONDITION
    operand1: Option<VariableName>,      // OPERAND1
    operand2: Option<VariableName>,      // OPERAND2
    operator: Option<ConditionOperator>, // OPERATOR
}

impl CnvType for Condition {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "CONDITION"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ConditionOperator::parse)
            .transpose()?;
        Ok(Self {
            operand1,
            operand2,
            operator,
        })
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
pub struct ComplexCondition {
    // COMPLEXCONDITION
    operand1: Option<ConditionName>,            // OPERAND1
    operand2: Option<ConditionName>,            // OPERAND2
    operator: Option<ComplexConditionOperator>, // OPERATOR
}

impl CnvType for ComplexCondition {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "COMPLEXCONDITION"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ComplexConditionOperator::parse)
            .transpose()?;
        Ok(Self {
            operand1,
            operand2,
            operator,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Dbl {
    // DOUBLE
    default: Option<f64>,    // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<f64>,      // VALUE
}

impl CnvType for Dbl {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "DOUBLE"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_f64)
            .transpose()?;
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties
            .remove("VALUE")
            .and_then(discard_if_empty)
            .map(parse_f64)
            .transpose()?;
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum EpisodeEvents {
    GoTo(String),
}

#[derive(Debug, Clone)]
pub struct Episode {
    // EPISODE
    pub author: Option<String>,                  // AUTHOR
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub description: Option<String>,             // DESCRIPTION
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub path: Option<String>,                    // PATH
    pub scenes: Option<Vec<SceneName>>,          // SCENES
    pub start_with: Option<SceneName>,           // STARTWITH
    pub version: Option<String>,                 // VERSION

    pub events: Vec<EpisodeEvents>,
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

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("GOTO") => {
                self.events.push(EpisodeEvents::GoTo(arguments[0].to_string()));
                None
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "PATH" => self.path.clone().map(|v| v.into()),
            "SCENES" => self.scenes.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
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
        Ok(Self {
            author,
            creation_time,
            description,
            last_modify_time,
            path,
            scenes,
            start_with,
            version,
            events: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    // EXPRESSION
    operand1: Option<VariableName>,       // OPERAND1
    operand2: Option<VariableName>,       // OPERAND2
    operator: Option<ExpressionOperator>, // OPERATOR
}

impl CnvType for Expression {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "EXPRESSION"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ExpressionOperator::parse)
            .transpose()?;
        Ok(Self {
            operand1,
            operand2,
            operator,
        })
    }
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

#[derive(Debug, Clone)]
pub struct Font {
    // FONT
    defs: HashMap<FontDef, Option<String>>,
}

lazy_static! {
    static ref FONT_DEF_REGEX: Regex = Regex::new(r"^DEF_(\w+)_(\w+)_(\d+)$").unwrap();
}

impl CnvType for Font {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "FONT"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let defs: HashMap<FontDef, Option<String>> = properties
            .into_iter()
            .filter_map(|(k, v)| {
                FONT_DEF_REGEX.captures(k.as_ref()).map(|m| {
                    (
                        FontDef {
                            family: m[1].to_owned(),
                            style: m[2].to_owned(),
                            size: m[3].parse().unwrap(),
                        },
                        Some(v),
                    )
                })
            })
            .collect();
        Ok(Self { defs })
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    // GROUP
}

impl CnvType for Group {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "GROUP"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(_properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        Ok(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    // IMAGE
    pub as_button: Option<bool>,                    // ASBUTTON
    pub filename: Option<String>,                   // FILENAME
    pub flush_after_played: Option<bool>,           // FLUSHAFTERPLAYED
    pub monitor_collision: Option<bool>,            // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>,      // MONITORCOLLISIONALPHA
    pub preload: Option<bool>,                      // PRELOAD
    pub priority: Option<i32>,                      // PRIORITY
    pub release: Option<bool>,                      // RELEASE
    pub to_canvas: Option<bool>,                    // TOCANVAS
    pub visible: Option<bool>,                      // VISIBLE
    pub on_init: Option<Arc<IgnorableProgram>>,     // ONINIT signal

    pub events: Vec<GraphicsEvents>,
}

impl CnvType for Image {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "IMAGE"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONCLICK"
                | "ONCOLLISION"
                | "ONCOLLISIONFINISHED"
                | "ONDONE"
                | "ONFOCUSOFF"
                | "ONFOCUSON"
                | "ONINIT"
                | "ONRELEASE"
                | "ONSIGNAL"
        )
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        script_runner: &mut CnvRunner,
        context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("HIDE") => {
                self.events.push(GraphicsEvents::Hide);
                None
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.on_init.as_ref() {
                    v.run(script_runner, context)
                }
                None
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "FILENAME" => self.filename.clone().map(|v| v.into()),
            "PRIORITY" => self.priority.map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let as_button = properties
            .remove("ASBUTTON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let release = properties
            .remove("RELEASE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self {
            as_button,
            filename,
            flush_after_played,
            monitor_collision,
            monitor_collision_alpha,
            preload,
            priority,
            release,
            to_canvas,
            visible,
            on_init,
            events: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Int {
    // INTEGER
    default: Option<i32>,    // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<i32>,      // VALUE
}

impl CnvType for Int {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "INTEGER"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties
            .remove("VALUE")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    // KEYBOARD
    keyboard: Option<String>, // KEYBOARD
}

impl CnvType for Keyboard {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "KEYBOARD"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let keyboard = properties.remove("KEYBOARD").and_then(discard_if_empty);
        Ok(Self { keyboard })
    }
}

#[derive(Debug, Clone)]
pub struct Mouse {
    // MOUSE
    mouse: Option<String>, // MOUSE
    raw: Option<i32>,      // RAW
}

impl CnvType for Mouse {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "MOUSE"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let mouse = properties.remove("MOUSE").and_then(discard_if_empty);
        let raw = properties
            .remove("RAW")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self { mouse, raw })
    }
}

#[derive(Debug, Clone)]
pub struct MultiArray {
    // MULTIARRAY
    dimensions: Option<i32>, // DIMENSIONS
}

impl CnvType for MultiArray {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "MULTIARRAY"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let dimensions = properties
            .remove("DIMENSIONS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self { dimensions })
    }
}

#[derive(Debug, Clone)]
pub struct Music {
    // MUSIC
    filename: Option<String>, // FILENAME
}

impl CnvType for Music {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "MUSIC"
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
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        Ok(Self { filename })
    }
}

#[derive(Debug, Clone)]
pub struct Random {
    // RAND
}

impl CnvType for Random {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "RANDOM"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(_properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        Ok(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    // SCENE
    pub author: Option<String>,                  // AUTHOR
    pub background: Option<String>,              // BACKGROUND
    pub coauthors: Option<String>,               // COAUTHORS
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub deamon: Option<bool>,                    // DEAMON
    pub description: Option<String>,             // DESCRIPTION
    pub dlls: Option<Vec<String>>,               // DLLS
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub music: Option<String>,                   // MUSIC
    pub path: Option<String>,                    // PATH
    pub version: Option<String>,                 // VERSION
}

impl CnvType for Scene {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SCENE"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "BACKGROUND" => self.background.clone().map(|v| v.into()),
            "PATH" => self.path.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let background = properties
            .remove("BACKGROUND")
            .and_then(discard_if_empty)
            .and_then(|s| if s.is_empty() { None } else { Some(s) });
        let coauthors = properties.remove("COAUTHORS").and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let deamon = properties
            .remove("DEAMON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let dlls = properties
            .remove("DLLS")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let music = properties.remove("MUSIC").and_then(discard_if_empty);
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        Ok(Self {
            author,
            background,
            coauthors,
            creation_time,
            deamon,
            description,
            dlls,
            last_modify_time,
            music,
            path,
            version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    // SEQUENCE
    filename: Option<String>, // FILENAME
}

impl CnvType for Sequence {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SEQUENCE"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONDONE" | "ONFINISHED" | "ONINIT" | "ONSIGNAL" | "ONSTARTED"
        )
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        Ok(Self { filename })
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    // SOUND
    filename: Option<String>,         // FILENAME
    flush_after_played: Option<bool>, // FLUSHAFTERPLAYED
    preload: Option<bool>,            // PRELOAD
}

impl CnvType for Sound {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SOUND"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONDONE" | "ONFINISHED" | "ONINIT" | "ONRESUMED" | "ONSIGNAL" | "ONSTARTED"
        )
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            filename,
            flush_after_played,
            preload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Str {
    // STRING
    default: Option<String>, // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<String>,   // VALUE
}

impl CnvType for Str {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "STRING"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let default = properties.remove("DEFAULT").and_then(discard_if_empty);
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties.remove("VALUE").and_then(discard_if_empty);
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
}

lazy_static! {
    static ref STRUCT_FIELDS_REGEX: Regex = Regex::new(r"^(\w+)<(\w+)>$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Struct {
    // STRUCT
    fields: Option<Vec<(String, TypeName)>>,
}

impl CnvType for Struct {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "STRUCT"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let fields = properties
            .remove("FIELDS")
            .and_then(discard_if_empty)
            .map(|s| {
                s.split(',')
                    .map(|f| {
                        let m = STRUCT_FIELDS_REGEX.captures(f).unwrap();
                        (m[1].to_owned(), m[2].to_owned())
                    })
                    .collect()
            });
        Ok(Self { fields })
    }
}

#[derive(Debug, Clone)]
pub struct System {
    // SYSTEM
    system: Option<String>, // SYSTEM
}

impl CnvType for System {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SYSTEM"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let system = properties.remove("SYSTEM").and_then(discard_if_empty);
        Ok(Self { system })
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    // TEXT
    font: Option<FontName>,                // FONT
    horizontal_justify: Option<bool>,      // HJUSTIFY
    hypertext: Option<bool>,               // HYPERTEXT
    monitor_collision: Option<bool>,       // MONITORCOLLISION
    monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    priority: Option<i32>,                 // PRIORITY
    rect: Option<Rect>,                    // RECT
    text: Option<String>,                  // TEXT
    to_canvas: Option<bool>,               // TOCANVAS
    visible: Option<bool>,                 // VISIBLE
    vertical_justify: Option<bool>,        // VJUSTIFY
}

impl CnvType for Text {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "TEXT"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let font = properties.remove("FONT").and_then(discard_if_empty);
        let horizontal_justify = properties
            .remove("HJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let hypertext = properties
            .remove("HYPERTEXT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let rect = properties
            .remove("RECT")
            .and_then(discard_if_empty)
            .map(parse_rect)
            .transpose()?;
        let text = properties.remove("TEXT").and_then(discard_if_empty);
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let vertical_justify = properties
            .remove("VJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            font,
            horizontal_justify,
            hypertext,
            monitor_collision,
            monitor_collision_alpha,
            priority,
            rect,
            text,
            to_canvas,
            visible,
            vertical_justify,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Timer {
    // TIMER
    elapse: Option<i32>,   // ELAPSE
    enabled: Option<bool>, // ENABLED
    ticks: Option<i32>,    // TICKS
}

impl CnvType for Timer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "TIMER"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(name, "ONDONE" | "ONINIT" | "ONSIGNAL" | "ONTICK")
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _script_runner: &mut CnvRunner,
        _context: &mut RunnerContext,
    ) -> Option<CnvValue> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(mut properties: HashMap<String, String>) -> Result<Self, TypeParsingError> {
        let elapse = properties
            .remove("ELAPSE")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let enabled = properties
            .remove("ENABLED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let ticks = properties
            .remove("TICKS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self {
            elapse,
            enabled,
            ticks,
        })
    }
}
