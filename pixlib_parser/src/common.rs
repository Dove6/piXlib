use std::{
    cell::{Ref, RefMut},
    error::Error,
    fmt::Display,
    ops::Add,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use log::{error, info, trace, warn};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Position {
    pub character: usize,
    pub line: usize,
    pub column: usize,
}

// FIXME: this is plainly invalid
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

impl<T> DroppableRefMut for RwLockWriteGuard<'_, T> {}
impl<T> DroppableRefMut for RwLockReadGuard<'_, T> {}

pub trait RemoveSearchable<T> {
    fn remove_found<P: FnMut(&T) -> bool>(&mut self, predicate: P) -> Option<T>;
}

impl<T> RemoveSearchable<T> for Vec<T> {
    fn remove_found<P: FnMut(&T) -> bool>(&mut self, predicate: P) -> Option<T> {
        self.iter().position(predicate).map(|idx| self.remove(idx))
    }
}

pub fn add_tuples<TR, TL: Add<TR>>(
    a: (TL, TL),
    b: (TR, TR),
) -> (
    <TL as std::ops::Add<TR>>::Output,
    <TL as std::ops::Add<TR>>::Output,
) {
    (a.0 + b.0, a.1 + b.1)
}

pub fn pair_u32_to_usize(pair: (u32, u32)) -> (usize, usize) {
    (pair.0 as usize, pair.1 as usize)
}

pub fn pair_u32_to_isize(pair: (u32, u32)) -> (isize, isize) {
    (pair.0 as isize, pair.1 as isize)
}

pub fn pair_i32_to_isize(pair: (i32, i32)) -> (isize, isize) {
    (pair.0 as isize, pair.1 as isize)
}

pub trait LoggableToOption<T> {
    fn ok_or_trace(self) -> Option<T>;
    fn ok_or_info(self) -> Option<T>;
    fn ok_or_warn(self) -> Option<T>;
    fn ok_or_error(self) -> Option<T>;
}

impl<TOk, TErr: Display> LoggableToOption<TOk> for Result<TOk, TErr> {
    fn ok_or_trace(self) -> Option<TOk> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                trace!("{}", e);
                None
            }
        }
    }

    fn ok_or_info(self) -> Option<TOk> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                info!("{}", e);
                None
            }
        }
    }

    fn ok_or_warn(self) -> Option<TOk> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                warn!("{}", e);
                None
            }
        }
    }

    fn ok_or_error(self) -> Option<TOk> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                error!("{}", e);
                None
            }
        }
    }
}

pub enum OkResult<TOk, TErr> {
    NoError(TOk),
    WithError(TOk, TErr),
}

use OkResult::{NoError, WithError};

impl<TOk, TErr> OkResult<TOk, TErr> {
    pub fn into_result(self) -> Result<TOk, TErr> {
        self.into()
    }

    pub fn and_then<TOk2>(
        self,
        mut f: impl FnMut(TOk) -> OkResult<TOk2, TErr>,
    ) -> OkResult<TOk2, TErr> {
        match self {
            NoError(v) => f(v),
            WithError(v, e) => match f(v) {
                NoError(w) => WithError(w, e),
                WithError(w, _) => WithError(w, e),
            },
        }
    }
}

impl<TOk, TErr> From<OkResult<TOk, TErr>> for Result<TOk, TErr> {
    fn from(value: OkResult<TOk, TErr>) -> Self {
        match value {
            NoError(v) => Ok(v),
            WithError(_, e) => Err(e),
        }
    }
}

impl<TOk, TErr: Display> LoggableToOption<TOk> for OkResult<TOk, TErr> {
    fn ok_or_trace(self) -> Option<TOk> {
        match self {
            NoError(value) => Some(value),
            WithError(value, e) => {
                trace!("{}", e);
                Some(value)
            }
        }
    }

    fn ok_or_info(self) -> Option<TOk> {
        match self {
            NoError(value) => Some(value),
            WithError(value, e) => {
                info!("{}", e);
                Some(value)
            }
        }
    }

    fn ok_or_warn(self) -> Option<TOk> {
        match self {
            NoError(value) => Some(value),
            WithError(value, e) => {
                warn!("{}", e);
                Some(value)
            }
        }
    }

    fn ok_or_error(self) -> Option<TOk> {
        match self {
            NoError(value) => Some(value),
            WithError(value, e) => {
                error!("{}", e);
                Some(value)
            }
        }
    }
}
