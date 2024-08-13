use std::{
    cell::{Ref, RefMut},
    error::Error,
    fmt::Display,
    ops::Add,
};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Position {
    pub character: usize,
    pub line: usize,
    pub column: usize,
}

// TODO: this is plainly invalid
impl Add<usize> for &Position {
    type Output = Position;

    fn add(self, rhs: usize) -> Self::Output {
        Position {
            character: self.character + rhs,
            column: self.column + rhs,
            line: self.line,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (line {}, col {})",
            self.character, self.line, self.column
        )
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            character: 0,
            line: 1,
            column: 1,
        }
    }
}

impl Position {
    pub fn with_incremented_line(&self, newline_length: usize) -> Self {
        Self {
            character: self.character + newline_length,
            line: self.line + 1,
            ..Default::default()
        }
    }

    pub fn with_incremented_column(&self) -> Self {
        Self {
            character: self.character + 1,
            line: self.line,
            column: self.column + 1,
        }
    }

    pub fn assign(&mut self, other: Self) -> Self {
        self.character = other.character;
        self.line = other.line;
        self.column = other.column;
        other
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Bounds {
    pub start: Position,
    pub end: Position,
}

impl Bounds {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn unit(position: Position) -> Self {
        Self {
            start: position,
            end: position,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WithPosition<T> {
    pub value: T,
    pub position: Position,
}

impl<T> WithPosition<T> {
    pub fn new(value: T, position: Position) -> Self {
        Self { value, position }
    }
}

pub trait MultiModeLexer {
    /// The type of the tokenization modes available.
    type Modes;

    /// Puts a new tokenization mode on the top of stack, making it the current one.
    fn push_mode(&mut self, mode: Self::Modes);

    /// Pops the current mode tokenization off the stack, switching back to the previous one.
    fn pop_mode(&mut self) -> Result<Self::Modes, &'static str>;

    /// Returns an immutable reference to the current tokenization mode.
    fn get_mode(&self) -> &Self::Modes;
}

pub trait ErrorHandler<E>: FnMut(E) + std::fmt::Debug {}

#[derive(Debug)]
pub struct Token<T> {
    pub value: T,
    pub bounds: Bounds,
    pub had_errors: bool,
}

pub trait Issue: Error {
    fn kind(&self) -> IssueKind;
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum IssueKind {
    Fatal,
    Error,
    Warning,
}

pub trait IssueHandler<I: Issue>: std::fmt::Debug {
    fn handle(&mut self, issue: I);
}

#[derive(Debug)]
pub struct IssueManager<I: Issue> {
    had_errors: bool,
    had_fatal: bool,
    handler: Option<Box<dyn IssueHandler<I> + Send + Sync>>,
}

impl<I: Issue> Default for IssueManager<I> {
    fn default() -> Self {
        Self {
            had_errors: false,
            had_fatal: false,
            handler: None,
        }
    }
}

impl<I: Issue> IssueManager<I> {
    pub fn had_errors(&self) -> bool {
        self.had_errors
    }

    pub fn clear_had_errors(&mut self) {
        self.had_errors = false;
    }

    pub fn had_fatal(&self) -> bool {
        self.had_fatal
    }

    pub fn set_handler(&mut self, handler: Box<dyn IssueHandler<I> + Send + Sync>) {
        self.handler = Some(handler);
    }

    pub fn emit_issue(&mut self, issue: I) {
        if matches!(issue.kind(), IssueKind::Fatal | IssueKind::Error) {
            self.had_errors = true;
        }
        if issue.kind() == IssueKind::Fatal {
            self.had_fatal = true;
        }
        if let Some(handler) = self.handler.as_mut() {
            handler.handle(issue)
        };
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub trait DroppableRefMut {
    fn use_and_drop<R>(self, mut f: impl FnMut(&Self) -> R) -> R
    where
        Self: Sized,
    {
        let r = f(&self);
        std::mem::drop(self);
        r
    }

    fn use_and_drop_mut<R>(mut self, f: impl FnOnce(&mut Self) -> R) -> R
    where
        Self: Sized,
    {
        let ret = f(&mut self);
        std::mem::drop(self);
        ret
    }
}

impl<T> DroppableRefMut for RefMut<'_, T> {}
impl<T> DroppableRefMut for Ref<'_, T> {}
