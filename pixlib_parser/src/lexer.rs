use crate::common::{Bounds, PositionalIterator, Element};

pub struct CnvLexer<I: PositionalIterator<Item = char>> {
    input: I,
    max_lexeme_length: usize,
    pub current_element: Element<CnvToken>,
}

impl<I: PositionalIterator<Item = char>> CnvLexer<I> {
    pub fn new(input: I, max_lexeme_length: usize) -> Self {
        Self {
            input,
            max_lexeme_length,
            current_element: Element::BeforeStream,
        }
    }
}

impl<I: PositionalIterator<Item = char>> PositionalIterator for CnvLexer<I> {
    type Item = CnvToken;

    fn advance(&mut self) -> std::io::Result<crate::common::Element<Self::Item>> {
        todo!()
    }

    fn get_current_element(&self) -> &crate::common::Element<Self::Item> {
        &self.current_element
    }
}

fn match_keyword_or_identifier_or_number() -> ! {
    todo!()
}

pub enum CnvToken {
    EndOfText,
    Unknown(String),

    LiteralString(String),
    LiteralInteger(i32),
    LiteralFloat(f64),

    KeywordThis,
    KeywordTrue,
    KeywordFalse,

    OperatorPlus,
    OperatorMinus,
    OperatorAsterisk,
    OperatorAt,
    OperatorPercent,
    OperatorCaret,
    Comma,
    Hash,
    Bang,
    Semicolon,
    At,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,

    Identifier(String),
}
