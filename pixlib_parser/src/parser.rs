use std::iter::Peekable;

use thiserror::Error;

use crate::{
    common::{Issue, IssueKind, IssueManager, Token},
    lexer::{CnvToken, LexerFatal},
};

type ParserInput = Result<Token<CnvToken>, LexerFatal>;

#[derive(Debug, Clone)]
pub struct IgnorableProgram {
    pub ignored: bool,
    pub value: Program,
}

#[derive(Debug, Clone)]
pub enum Program {
    Resolvable(String),
    Block(Vec<IgnorableStatement>),
}

#[derive(Debug, Clone)]
pub struct IgnorableStatement {
    pub ignored: bool,
    pub value: Statement,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Invocation {
        parent: Option<String>,
        name: String,
        arguments: Vec<Expression>,
    },
    ExpressionStatement(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Resolvable(String),
    Operation(Box<Operation>),
    Block(Vec<IgnorableStatement>),
}

#[derive(Debug, Clone)]
pub enum Operation {
    Addition(Expression, Expression),
    Multiplication(Expression, Expression),
    Subtraction(Expression, Expression),
    IntegerDivision(Expression, Expression),
    Remainder(Expression, Expression),
}

#[derive(Error, Debug, Clone)]
pub enum ParserFatal {}

#[derive(Error, Debug, Clone)]
pub enum ParserError {}

#[derive(Error, Debug, Clone)]
pub enum ParserWarning {}

#[derive(Error, Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ParsingSettings {}

#[derive(Debug)]
pub struct CnvParser<I: Iterator<Item = ParserInput>> {
    _input: Peekable<I>,
    _issue_manager: IssueManager<ParserIssue>,
    _settings: ParsingSettings,
}

impl<I: Iterator<Item = ParserInput> + 'static> CnvParser<I> {
    pub fn parse(&mut self) -> Result<Option<IgnorableProgram>, ParserFatal> {
        Ok(None)
    }
}
