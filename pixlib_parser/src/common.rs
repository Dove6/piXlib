#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Position {
    pub character: usize,
    pub line: usize,
    pub column: usize,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locatable<T> {
    pub value: T,
    pub bounds: Bounds,
}

impl<T> Locatable<T> {
    pub fn new(value: T, bounds: Bounds) -> Self {
        Self { value, bounds }
    }
}

pub trait MultiModeLexer {
    /// The type of the tokenization modes available.
    type Modes;

    /// Puts a new tokenization mode on the top of stack, making it the current one.
    fn push_mode(&mut self, mode: Self::Modes);

    /// Pops the current mode tokenization off the stack, switching back to the previous one.
    fn pop_mode(&mut self) -> Self::Modes;

    /// Returns an immutable reference to the current tokenization mode.
    fn get_mode(&self) -> &Self::Modes;
}

pub struct ErrorManager<E> {
    pub encountered_error: bool,
    pub encountered_fatal: bool,
    pub error_handler: Option<Box<dyn FnMut(E)>>,
}

impl<E> Default for ErrorManager<E> {
    fn default() -> Self {
        Self {
            encountered_error: false,
            encountered_fatal: false,
            error_handler: None,
        }
    }
}

impl<E> ErrorManager<E> {
    pub fn emit_error(&mut self, error: E) {
        self.encountered_error = true;
        if let Some(error_handler) = self.error_handler.as_mut() {
            error_handler(error)
        };
    }
}
