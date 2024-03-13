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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Element<T> {
    BeforeStream,
    WithinStream { element: T, bounds: Bounds },
    AfterStream,
}

impl<T> Element<T> {
    pub fn unwrap(self) -> T {
        match self {
            Self::WithinStream { element, bounds: _ } => element,
            _ => panic!(),
        }
    }

    pub fn get_element(&self) -> Option<&T> {
        match self {
            Self::WithinStream { element, bounds: _ } => Some(element),
            _ => None,
        }
    }

    pub fn get_bounds(&self) -> Option<Bounds> {
        match self {
            Self::WithinStream { element: _, bounds } => Some(*bounds),
            _ => None,
        }
    }
}

pub trait PositionalIterator {
    /// The type of the elements being iterated over.
    type Item;

    /// Advances the iterator and returns the current (superseded) element.
    fn advance(&mut self) -> std::io::Result<Element<Self::Item>>;

    /// Returns an immutable reference to the current element.
    fn get_current_element(&self) -> &Element<Self::Item>;
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
