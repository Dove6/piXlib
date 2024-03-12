use crate::common::Bounds;

pub struct CnvLexer {
    pub current_token: CnvToken,
    pub current_bounds: Bounds,
    max_lexeme_length: usize,
}

impl CnvLexer {
    pub fn advance(&mut self) -> std::io::Result<()> {
        Ok(())
    }
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
