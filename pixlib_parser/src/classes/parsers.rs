use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    sync::Arc,
    vec::IntoIter,
};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use lalrpop_util::ParseError;
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::{
    ast::{ParsedScript, ParserFatal, ParserIssue},
    common::{Issue, IssueHandler, IssueManager, Position},
    lexer::{CnvLexer, CnvToken},
    parser::CodeParser,
    scanner::CnvScanner,
};

#[derive(Debug)]
pub enum PropertyValue {
    Boolean(bool),
    Integer(i32),
    Double(f64),
    String(String),
    List(Vec<String>),
    Rect(Rect),
    Time(DateTime<Utc>),
    Code(Arc<ParsedScript>),
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

impl From<Arc<ParsedScript>> for PropertyValue {
    fn from(value: Arc<ParsedScript>) -> Self {
        PropertyValue::Code(value)
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
    pub family: String,
    pub style: String,
    pub size: usize,
}

lazy_static! {
    pub static ref STRUCT_FIELDS_REGEX: Regex = Regex::new(r"^(\w+)<(\w+)>$").unwrap();
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

pub fn parse_bool(s: String) -> Result<bool, TypeParsingError> {
    match s.as_ref() {
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        _ => Err(TypeParsingError::InvalidBoolLiteral(s)),
    }
}

pub fn parse_i32(s: String) -> Result<i32, TypeParsingError> {
    s.parse().map_err(TypeParsingError::InvalidIntegerLiteral)
}

pub fn parse_f64(s: String) -> Result<f64, TypeParsingError> {
    s.parse().map_err(TypeParsingError::InvalidFloatingLiteral)
}

pub fn parse_datetime(_s: String) -> Result<DateTime<Utc>, TypeParsingError> {
    Ok(DateTime::default()) // TODO: parse date
}

pub fn parse_comma_separated(s: String) -> Result<Vec<String>, TypeParsingError> {
    Ok(s.split(',').map(|s| s.trim().to_owned()).collect())
}

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        eprintln!("{:?}", issue);
    }
}

pub fn parse_program(s: String) -> Result<Arc<ParsedScript>, TypeParsingError> {
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

pub fn parse_rect(s: String) -> Result<Rect, TypeParsingError> {
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

pub fn discard_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
