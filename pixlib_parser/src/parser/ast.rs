use thiserror::Error;

use crate::{
    common::{Issue, IssueKind, Position},
    lexer::LexerFatal,
};

pub type ParsedScript = IgnorableExpression;

#[derive(Debug, Clone)]
pub struct IgnorableExpression {
    pub ignored: bool,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub enum Statement {
    ExpressionStatement(IgnorableExpression),
}

#[derive(Debug, Clone)]
pub struct Invocation {
    pub parent: Option<Expression>,
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    LiteralBool(bool),
    LiteralNull,
    SelfReference,
    Identifier(String),
    Invocation(Box<Invocation>),
    Parameter(String), // TODO: Parameter(usize)
    NameResolution(Box<Expression>),
    FieldAccess(Box<Expression>, String),
    Operation(Box<Expression>, Vec<(Operation, Expression)>),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Operation {
    Addition,
    Multiplication,
    Subtraction,
    Division,
    Remainder,
}

#[derive(Error, Debug)]
pub enum ParserFatal {
    #[error("Lexer error")]
    LexerError {
        #[from]
        source: LexerFatal,
    },
}

#[derive(Error, Debug, Clone)]
pub enum ParserError {
    #[error("Expected argument at {0}")]
    ExpectedArgument(Position),
}

#[derive(Error, Debug, Clone)]
pub enum ParserWarning {}

#[derive(Error, Debug)]
pub enum ParserIssue {
    #[error("Fatal error: {0}")]
    Fatal(ParserFatal),
    #[error("Error: {0}")]
    Error(ParserError),
    #[error("Warning: {0}")]
    Warning(ParserWarning),
}

impl Issue for ParserIssue {
    fn kind(&self) -> IssueKind {
        match *self {
            Self::Fatal(_) => IssueKind::Fatal,
            Self::Error(_) => IssueKind::Error,
            Self::Warning(_) => IssueKind::Warning,
        }
    }
}

impl From<ParserFatal> for ParserIssue {
    fn from(value: ParserFatal) -> Self {
        Self::Fatal(value)
    }
}

impl From<ParserError> for ParserIssue {
    fn from(value: ParserError) -> Self {
        Self::Error(value)
    }
}

impl From<ParserWarning> for ParserIssue {
    fn from(value: ParserWarning) -> Self {
        Self::Warning(value)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParsingSettings {}
