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

pub enum Element<T> {
    BeforeStart,
    WithinStream { element: T, bounds: Bounds },
    AfterEnd,
}

pub trait PositionalIterator {
    /// The type of the elements being iterated over.
    type Item;

    /// Advances the iterator and returns the current (superseded) element.
    fn advance(&mut self) -> std::io::Result<Element<Self::Item>>;

    /// Returns an immutable reference to the current element.
    fn get_current_element(&self) -> Element<&Self::Item>;

    /// Returns [`Bounds`] describing the position of the current element in the underlying [`char`] stream.
    fn get_current_bounds(&self) -> Bounds;
}
