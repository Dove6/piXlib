use thiserror::Error;

use crate::{
    ast::ParserFatal,
    common::{Bounds, Issue, IssueKind, IssueManager, Position, Spanned},
};
use std::iter::Peekable;

type LexerInput = Spanned<char, Position, std::io::Error>;
type LexerOutput = Spanned<CnvToken, Position, ParserFatal>;

#[derive(Error, Debug)]
pub enum LexerFatal {
    #[error("IO error at {position}")]
    IoError {
        position: Position,
        source: std::io::Error,
    },
}

#[derive(Error, Debug, Clone)]
pub enum LexerError {
    #[error("Unexpected character '{character}' at {position}")]
    UnexpectedCharacter { position: Position, character: char },
    #[error("Lexeme length over limit ({max_allowed_len}) at {} to {}", bounds.start, bounds.end)]
    LexemeTooLong {
        bounds: Bounds,
        max_allowed_len: usize,
    },
}

#[derive(Error, Debug, Clone)]
pub enum LexerWarning {}

#[derive(Error, Debug)]
pub enum LexerIssue {
    #[error("Fatal error: {0}")]
    Fatal(LexerFatal),
    #[error("Error: {0}")]
    Error(LexerError),
    #[error("Warning: {0}")]
    Warning(LexerWarning),
}

impl Issue for LexerIssue {
    fn kind(&self) -> IssueKind {
        match *self {
            Self::Fatal(_) => IssueKind::Fatal,
            Self::Error(_) => IssueKind::Error,
            Self::Warning(_) => IssueKind::Warning,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenizationSettings {
    max_lexeme_length: usize,
}

impl Default for TokenizationSettings {
    fn default() -> Self {
        Self {
            max_lexeme_length: i32::MAX as usize,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct LexerState {
    pub brace_level: usize,
    pub bracket_level: usize,
    pub parenthesis_level: usize,
    pub expecting_arguments: bool,
}

#[derive(Debug)]
pub struct CnvLexer<I: Iterator<Item = LexerInput>> {
    input: Peekable<I>,
    issue_manager: IssueManager<LexerIssue>,
    settings: TokenizationSettings,
    state: LexerState,
    next_position: Position,
}

impl<I: Iterator<Item = LexerInput> + 'static> CnvLexer<I> {
    pub fn new(
        input: I,
        settings: TokenizationSettings,
        issue_manager: IssueManager<LexerIssue>,
    ) -> Self {
        Self {
            input: input.peekable(),
            issue_manager,
            settings,
            state: LexerState::default(),
            next_position: Position::default(),
        }
    }
}

impl<I: Iterator<Item = LexerInput> + 'static> Iterator for CnvLexer<I> {
    type Item = LexerOutput;

    fn next(&mut self) -> Option<Self::Item> {
        self.issue_manager.clear_had_errors();
        self.input.peek()?;
        if self.issue_manager.had_fatal() {
            return None;
        }
        while self
            .input
            .next_if(|result| result.as_ref().is_ok_and(|(_, c, _)| c.is_whitespace()))
            .is_some()
        {}
        match self.input.next() {
            Some(Ok((pos, '@', next_pos))) => {
                self.state.expecting_arguments = true;
                Some(Ok((pos, CnvToken::At, self.next_position.assign(next_pos))))
            }
            Some(Ok((pos, '^', next_pos))) => {
                self.state.expecting_arguments = true;
                Some(Ok((
                    pos,
                    CnvToken::Caret,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, '|', next_pos))) => Some(Ok((
                pos,
                CnvToken::Pipe,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, ',', next_pos))) => Some(Ok((
                pos,
                CnvToken::Comma,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, '$', next_pos))) => Some(Ok((
                pos,
                CnvToken::Dollar,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, '!', next_pos))) => Some(Ok((
                pos,
                CnvToken::Bang,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, ';', next_pos))) => {
                self.state.expecting_arguments = false;
                Some(Ok((
                    pos,
                    CnvToken::Semicolon,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, '(', next_pos))) => {
                self.state.expecting_arguments = false;
                self.state.parenthesis_level += 1; // TODO: check limits
                Some(Ok((
                    pos,
                    CnvToken::LeftParenthesis,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, ')', next_pos))) => {
                self.state.parenthesis_level = self.state.parenthesis_level.saturating_sub(1);
                Some(Ok((
                    pos,
                    CnvToken::RightParenthesis,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, '[', next_pos))) => {
                self.state.bracket_level += 1;
                Some(Ok((
                    pos,
                    CnvToken::LeftBracket,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, ']', next_pos))) => {
                self.state.bracket_level = self.state.bracket_level.saturating_sub(1);
                Some(Ok((
                    pos,
                    CnvToken::RightBracket,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, '{', next_pos))) => {
                self.state.brace_level += 1;
                Some(Ok((
                    pos,
                    CnvToken::LeftBrace,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, '}', next_pos))) => {
                self.state.brace_level = self.state.brace_level.saturating_sub(1);
                Some(Ok((
                    pos,
                    CnvToken::RightBrace,
                    self.next_position.assign(next_pos),
                )))
            }
            Some(Ok((pos, '+', next_pos))) => Some(Ok((
                pos,
                CnvToken::Plus,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, '-', next_pos))) if self.state.bracket_level > 0 => Some(Ok((
                pos,
                CnvToken::Minus,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, '*', next_pos))) => Some(Ok((
                pos,
                CnvToken::Asterisk,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, '%', next_pos))) => Some(Ok((
                pos,
                CnvToken::Percent,
                self.next_position.assign(next_pos),
            ))),
            Some(Ok((pos, character, next_pos))) => {
                self.next_position = next_pos;
                let mut lexeme = String::new();
                let mut relative_parenthesis_level: usize = 0;
                let mut relative_bracket_level: usize = 0;
                let mut relative_brace_level: usize = 0;
                lexeme.push(character);
                let mut length_exceeded = false;
                while let Some(triple) = self.input.next_if(|result| {
                    result.as_ref().is_ok_and(|(_, c, _)| match c {
                        '|' | '$' => false,
                        '^' | ';' => relative_brace_level > 0,
                        '+' | '-' | '*' | '@' | '%' => self.state.bracket_level == 0,
                        ',' => relative_parenthesis_level > 0,
                        '(' => {
                            relative_parenthesis_level += 1;
                            !self.state.expecting_arguments
                        }
                        ')' => {
                            if relative_parenthesis_level == 0 {
                                false
                            } else {
                                relative_parenthesis_level -= 1;
                                true
                            }
                        }
                        '[' => {
                            relative_bracket_level += 1;
                            true
                        }
                        ']' => {
                            if relative_bracket_level == 0 {
                                false
                            } else {
                                relative_bracket_level -= 1;
                                true
                            }
                        }
                        '{' => {
                            relative_brace_level += 1;
                            true
                        }
                        '}' => {
                            if relative_brace_level == 0 {
                                false
                            } else {
                                relative_brace_level -= 1;
                                true
                            }
                        }
                        _ => true,
                    })
                }) {
                    if lexeme.len() >= self.settings.max_lexeme_length {
                        length_exceeded = true;
                    }
                    let triple = triple.unwrap();
                    let c = triple.1;
                    self.next_position = triple.2;
                    if !length_exceeded {
                        lexeme.push(c);
                    }
                }
                if length_exceeded {
                    self.issue_manager
                        .emit_issue(LexerIssue::Error(LexerError::LexemeTooLong {
                            bounds: Bounds::new(pos, next_pos),
                            max_allowed_len: self.settings.max_lexeme_length,
                        }));
                } else if lexeme.eq_ignore_ascii_case("TRUE") {
                    // TODO: check if case should be ignored
                    return Some(Ok((pos, CnvToken::KeywordTrue, self.next_position)));
                } else if lexeme.eq_ignore_ascii_case("FALSE") {
                    // TODO: check if case should be ignored
                    return Some(Ok((pos, CnvToken::KeywordFalse, self.next_position)));
                } else if lexeme.eq_ignore_ascii_case("THIS") {
                    // TODO: check if case should be ignored
                    return Some(Ok((pos, CnvToken::KeywordThis, self.next_position)));
                }
                Some(Ok((pos, CnvToken::Identifier(lexeme), self.next_position)))
            }
            Some(Err(err)) => Some(Err(LexerFatal::IoError {
                position: self.next_position,
                source: err,
            }
            .into())),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CnvToken {
    Unexpected(char),

    Identifier(String),
    KeywordTrue,
    KeywordFalse,
    KeywordThis,
    Plus,
    Minus,
    Asterisk,
    At,
    Percent,
    Caret,
    Pipe,
    Comma,
    Dollar,
    Bang,
    Semicolon,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}
