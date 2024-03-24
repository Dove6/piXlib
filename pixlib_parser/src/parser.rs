use std::iter::Peekable;

use crate::{
    common::{ErrorManager, Locatable},
    lexer::CnvToken,
};

type ParserInput = Locatable<CnvToken>;

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

#[derive(Debug, Clone)]
pub enum ParserFatal {}

#[derive(Debug, Clone)]
pub enum ParserError {}

#[derive(Debug, Clone)]
pub enum ParserWarning {}

#[derive(Debug, Clone)]
pub enum ParserIssue {
    Fatal(ParserFatal),
    Error(ParserError),
    Warning(ParserWarning),
}

#[derive(Debug, Clone)]
pub struct ParsingSettings {}

#[derive(Debug)]
pub struct CnvParser<I: Iterator<Item = ParserInput>> {
    _input: Peekable<I>,
    _error_manager: ErrorManager<ParserError>,
    _settings: ParsingSettings,
}

impl<I: Iterator<Item = ParserInput> + 'static> CnvParser<I> {
    pub fn parse(&mut self) -> Result<Option<IgnorableProgram>, ParserFatal> {
        Ok(None)
    }
}
