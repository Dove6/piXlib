use lazy_static::lazy_static;

use crate::common::{Bounds, Locatable, MultiModeLexer, Position, WithPosition};
use std::{collections::HashMap, iter::Peekable};

type LexerInput = WithPosition<std::io::Result<char>>;
type LexerOutput = Locatable<CnvToken>;
type LexerErrorHandler = Box<dyn FnMut(LexerError)>;
type LexemeMatcher<I> =
    fn(&mut I, &mut Option<LexerErrorHandler>, &TokenizationSettings) -> Option<LexerOutput>;

pub enum LexerError {
    UnexpectedCharacter {
        position: Position,
        character: char,
    },
    LexemeTooLong {
        bounds: Bounds,
        max_allowed_len: usize,
    },
    IoError {
        position: Position,
        error: std::io::Error,
    },
}

pub enum CnvTokenizationModes {
    General,
    Operation,
}

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

pub struct CnvLexer<I: Iterator<Item = LexerInput>> {
    input: Peekable<I>,
    encountered_fatal: bool,
    error_handler: Option<LexerErrorHandler>,
    settings: TokenizationSettings,
    tokenization_mode_stack: Vec<CnvTokenizationModes>,
}

impl<I: Iterator<Item = LexerInput> + 'static> CnvLexer<I> {
    const GENERAL_MATCHERS: &'static [LexemeMatcher<Peekable<I>>] =
        &[match_resolvable, match_symbol];
    const OPERATION_MATCHERS: &'static [LexemeMatcher<Peekable<I>>] =
        &[match_operation_resolvable, match_operation_symbol];

    pub fn new(input: I, settings: TokenizationSettings) -> Self {
        Self {
            input: input.peekable(),
            encountered_fatal: false,
            error_handler: None,
            settings,
            tokenization_mode_stack: Vec::new(),
        }
    }

    #[inline]
    fn get_matchers(&self) -> &'static [LexemeMatcher<Peekable<I>>] {
        match self.get_mode() {
            CnvTokenizationModes::General => Self::GENERAL_MATCHERS,
            CnvTokenizationModes::Operation => Self::OPERATION_MATCHERS,
        }
    }
}

impl<I: Iterator<Item = LexerInput>> MultiModeLexer for CnvLexer<I> {
    type Modes = CnvTokenizationModes;

    fn push_mode(&mut self, mode: Self::Modes) {
        self.tokenization_mode_stack.push(mode);
    }

    fn pop_mode(&mut self) -> Self::Modes {
        if self.tokenization_mode_stack.is_empty() {
            panic!("Cannot pop elements off an empty stack!");
        }
        self.tokenization_mode_stack.pop().unwrap()
    }

    fn get_mode(&self) -> &Self::Modes {
        if self.tokenization_mode_stack.is_empty() {
            &Self::Modes::General
        } else {
            self.tokenization_mode_stack.last().unwrap()
        }
    }
}

impl<I: Iterator<Item = LexerInput> + 'static> Iterator for CnvLexer<I> {
    type Item = LexerOutput;

    fn next(&mut self) -> Option<Self::Item> {
        if self.encountered_fatal {
            return None;
        }
        match self
            .get_matchers()
            .iter()
            .find_map(|matcher| matcher(&mut self.input, &mut self.error_handler, &self.settings))
        {
            None => self.input.next().and_then(|p| match p.value {
                Ok(character) => {
                    if let Some(f) = self.error_handler.as_mut() {
                        f(LexerError::UnexpectedCharacter {
                            position: p.position,
                            character,
                        })
                    };
                    Some(Locatable {
                        value: CnvToken::Unexpected(character),
                        bounds: Bounds::unit(p.position),
                    })
                }
                Err(err) => {
                    if let Some(f) = self.error_handler.as_mut() {
                        f(LexerError::IoError {
                            position: p.position,
                            error: err,
                        })
                    };
                    None
                }
            }),
            matched_element => matched_element,
        }
    }
}

fn is_part_of_resolvable(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.' || c == '-'
}

fn match_resolvable(
    input: &mut Peekable<impl Iterator<Item = LexerInput>>,
    error_handler: &mut Option<LexerErrorHandler>,
    settings: &TokenizationSettings,
) -> Option<LexerOutput> {
    if !matches!(input.peek(), Some(WithPosition { value: Ok(c), .. }) if is_part_of_resolvable(*c))
    {
        return None;
    }
    let start_position = input.peek().unwrap().position;
    let mut end_position = start_position;
    let mut lexeme = String::new();
    while let Some(p) = input.next_if(|l| match &l.value {
        Ok(c) => is_part_of_resolvable(*c),
        Err(_) => true,
    }) {
        end_position = p.position;
        match p.value {
            Ok(character) => {
                if lexeme.len() >= settings.max_lexeme_length {
                    let mut lexeme_end = p.position;
                    while input
                        .next_if(|l| l.value.as_ref().is_ok_and(|c| is_part_of_resolvable(*c)))
                        .is_some()
                    {
                        lexeme_end = input.next().map(|l| l.position).unwrap_or(lexeme_end);
                    }
                    if let Some(f) = error_handler.as_mut() {
                        f(LexerError::LexemeTooLong {
                            bounds: Bounds::new(start_position, lexeme_end),
                            max_allowed_len: settings.max_lexeme_length,
                        })
                    };
                    break;
                }
                lexeme.push(character);
            }
            Err(err) => {
                if let Some(f) = error_handler.as_mut() {
                    f(LexerError::IoError {
                        position: p.position,
                        error: err,
                    })
                };
                // TODO: mark fatal error
                break;
            }
        }
    }
    Some(Locatable {
        value: CnvToken::Resolvable(lexeme),
        bounds: Bounds {
            start: start_position,
            end: end_position,
        },
    })
}

lazy_static! {
    static ref SYMBOL_MAPPING: HashMap<char, CnvToken> = [
        ('@', CnvToken::At),
        ('^', CnvToken::Caret),
        (',', CnvToken::Comma),
        ('!', CnvToken::Bang),
        (';', CnvToken::Semicolon),
        ('(', CnvToken::LeftParenthesis),
        (')', CnvToken::RightParenthesis),
        ('[', CnvToken::LeftBracket),
        (']', CnvToken::RightBracket),
        ('{', CnvToken::LeftBrace),
        ('}', CnvToken::RightBrace),
    ]
    .into_iter()
    .collect();
}

fn match_symbol(
    input: &mut Peekable<impl Iterator<Item = LexerInput>>,
    _: &mut Option<LexerErrorHandler>,
    _: &TokenizationSettings,
) -> Option<LexerOutput> {
    input
        .next_if(|p| {
            p.value
                .as_ref()
                .is_ok_and(|c| SYMBOL_MAPPING.contains_key(c))
        })
        .map(|l| {
            LexerOutput::new(
                SYMBOL_MAPPING[&l.value.unwrap()].clone(),
                Bounds::unit(l.position),
            )
        })
}

fn is_part_of_operation_resolvable(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn match_operation_resolvable(
    input: &mut Peekable<impl Iterator<Item = LexerInput>>,
    error_handler: &mut Option<LexerErrorHandler>,
    settings: &TokenizationSettings,
) -> Option<LexerOutput> {
    if !matches!(input.peek(), Some(WithPosition { value: Ok(c), .. }) if is_part_of_operation_resolvable(*c))
    {
        return None;
    }
    let start_position = input.peek().unwrap().position;
    let mut end_position = start_position;
    let mut lexeme = String::new();
    while let Some(p) = input.next_if(|l| match &l.value {
        Ok(c) => is_part_of_operation_resolvable(*c),
        Err(_) => true,
    }) {
        end_position = p.position;
        match p.value {
            Ok(character) => {
                if lexeme.len() >= settings.max_lexeme_length {
                    let mut lexeme_end = p.position;
                    while input
                        .next_if(|l| {
                            l.value
                                .as_ref()
                                .is_ok_and(|c| is_part_of_operation_resolvable(*c))
                        })
                        .is_some()
                    {
                        lexeme_end = input.next().map(|l| l.position).unwrap_or(lexeme_end);
                    }
                    if let Some(f) = error_handler.as_mut() {
                        f(LexerError::LexemeTooLong {
                            bounds: Bounds::new(start_position, lexeme_end),
                            max_allowed_len: settings.max_lexeme_length,
                        })
                    };
                    break;
                }
                lexeme.push(character);
            }
            Err(err) => {
                if let Some(f) = error_handler.as_mut() {
                    f(LexerError::IoError {
                        position: p.position,
                        error: err,
                    })
                };
                // TODO: mark fatal error
                break;
            }
        }
    }
    Some(Locatable {
        value: CnvToken::Resolvable(lexeme),
        bounds: Bounds {
            start: start_position,
            end: end_position,
        },
    })
}

lazy_static! {
    static ref OPERATION_SYMBOL_MAPPING: HashMap<char, CnvToken> = [
        ('+', CnvToken::Plus),
        ('-', CnvToken::Minus),
        ('*', CnvToken::Asterisk),
        ('@', CnvToken::At),
        ('%', CnvToken::Percent),
        ('^', CnvToken::Caret),
        (',', CnvToken::Comma),
        ('(', CnvToken::LeftParenthesis),
        (')', CnvToken::RightParenthesis),
        ('[', CnvToken::LeftBracket),
        (']', CnvToken::RightBracket),
    ]
    .into_iter()
    .collect();
}

fn match_operation_symbol(
    input: &mut Peekable<impl Iterator<Item = LexerInput>>,
    _: &mut Option<LexerErrorHandler>,
    _: &TokenizationSettings,
) -> Option<LexerOutput> {
    input
        .next_if(|p| {
            p.value
                .as_ref()
                .is_ok_and(|c| OPERATION_SYMBOL_MAPPING.contains_key(c))
        })
        .map(|l| {
            LexerOutput::new(
                OPERATION_SYMBOL_MAPPING[&l.value.unwrap()].clone(),
                Bounds::unit(l.position),
            )
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CnvToken {
    Unknown,
    Unexpected(char),

    Resolvable(String),
    OperationResolvable(String),

    Plus,
    Minus,
    Asterisk,
    At,
    Percent,
    Caret,
    Comma,
    Bang,
    Semicolon,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}
