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
    common::Position,
    lexer::{CnvLexer, CnvToken},
    parser::{
        ast::{self, Invocation, ParsedScript, ParserFatal},
        imperative_parser::CodeParser,
    },
    scanner::CnvScanner,
};

use super::Rect;

#[derive(Debug, Clone)]
pub enum ReferenceRect {
    Literal(Rect),
    Reference(String),
}

impl Default for ReferenceRect {
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
            _ => Err(TypeParsingError::InvalidConditionOperator(s)),
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
            _ => Err(TypeParsingError::InvalidComplexConditionOperator(s)),
        }
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
            _ => Err(TypeParsingError::InvalidExpressionOperator(s)),
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
    #[error("Missing operator")]
    MissingOperator,
    #[error("Missing left operand")]
    MissingLeftOperand,
    #[error("Missing right operand")]
    MissingRightOperand,
    #[error("Missing dimension count")]
    MissingDimensionCount,
    #[error("Event handler not callable")]
    EventHandlerNotCallable,
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

pub fn parse_program(s: String) -> Result<Arc<ParsedScript>, TypeParsingError> {
    let scanner = CnvScanner::<IntoIter<_>>::new(s.chars().map(Ok).collect::<Vec<_>>().into_iter());
    let lexer = CnvLexer::new(scanner, Default::default(), Default::default());
    Ok(Arc::new(
        CodeParser::new().parse(&Default::default(), lexer)?,
    ))
}

pub fn parse_event_handler(s: String) -> Result<Arc<ParsedScript>, TypeParsingError> {
    let program = parse_program(s)?;
    match &program.value {
        ast::Expression::Invocation(_) | ast::Expression::Block(_) => Ok(program),
        identifier @ ast::Expression::Identifier(_) => Ok(Arc::new(ParsedScript {
            ignored: program.ignored,
            value: ast::Expression::Invocation(Box::new(Invocation {
                parent: Some(identifier.clone()),
                name: "RUNC".into(),
                arguments: Vec::new(),
            })),
        })),
        _ => Err(TypeParsingError::EventHandlerNotCallable),
    }
}

pub fn parse_rect(s: String) -> Result<ReferenceRect, TypeParsingError> {
    if s.contains(',') {
        s.split(',')
            .map(|s| s.parse().unwrap())
            .collect_tuple()
            .map(|t: (isize, isize, isize, isize)| ReferenceRect::Literal(t.into()))
            .ok_or(TypeParsingError::InvalidRectLiteral(s))
    } else {
        Ok(ReferenceRect::Reference(s))
    }
}

pub fn discard_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
