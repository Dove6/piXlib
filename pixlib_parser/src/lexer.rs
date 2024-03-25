use thiserror::Error;

use crate::common::{
    Bounds, Issue, IssueKind, IssueManager, MultiModeLexer, Position, Token, WithPosition,
};
use std::iter::Peekable;

type LexerInput = WithPosition<std::io::Result<char>>;
type LexerOutput = Result<Token<CnvToken>, LexerFatal>;
type LexemeMatcher<I> = fn(
    &mut Peekable<I>,
    &mut IssueManager<LexerIssue>,
    &TokenizationSettings,
) -> Option<LexerOutput>;

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
pub enum CnvTokenizationModes {
    General,
    Operation,
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

#[derive(Debug)]
pub struct CnvLexer<I: Iterator<Item = LexerInput>> {
    input: Peekable<I>,
    issue_manager: IssueManager<LexerIssue>,
    settings: TokenizationSettings,
    tokenization_mode_stack: Vec<CnvTokenizationModes>,
}

impl<I: Iterator<Item = LexerInput> + 'static> CnvLexer<I> {
    const GENERAL_MATCHERS: &'static [LexemeMatcher<I>] = &[
        matchers::general_mode::match_resolvable,
        matchers::general_mode::match_symbol,
    ];
    const OPERATION_MATCHERS: &'static [LexemeMatcher<I>] = &[
        matchers::operation_mode::match_resolvable,
        matchers::operation_mode::match_symbol,
    ];

    pub fn new(input: I, settings: TokenizationSettings) -> Self {
        Self {
            input: input.peekable(),
            issue_manager: IssueManager::default(),
            settings,
            tokenization_mode_stack: Vec::new(),
        }
    }

    #[inline]
    fn get_matchers(&self) -> &'static [LexemeMatcher<I>] {
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

    fn pop_mode(&mut self) -> Result<Self::Modes, &'static str> {
        if self.tokenization_mode_stack.is_empty() {
            Err("Cannot pop elements off an empty stack!")
        } else {
            Ok(self.tokenization_mode_stack.pop().unwrap())
        }
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
        self.issue_manager.clear_had_errors();
        self.input.peek()?;
        if self.issue_manager.had_fatal() {
            return None;
        }
        match self
            .get_matchers()
            .iter()
            .find_map(|matcher| matcher(&mut self.input, &mut self.issue_manager, &self.settings))
        {
            None => self.input.next().map(|p| match p.value {
                Ok(character) => {
                    self.issue_manager.emit_issue(LexerIssue::Error(
                        LexerError::UnexpectedCharacter {
                            position: p.position,
                            character,
                        },
                    ));
                    Ok(Token {
                        value: CnvToken::Unexpected(character),
                        bounds: Bounds::unit(p.position),
                        had_errors: self.issue_manager.had_errors(),
                    })
                }
                Err(err) => {
                    self.issue_manager
                        .emit_issue(LexerIssue::Fatal(LexerFatal::IoError {
                            position: p.position,
                            source: std::io::Error::from(err.kind()),
                        }));
                    Err(LexerFatal::IoError {
                        position: p.position,
                        source: err,
                    })
                }
            }),
            matched_element => matched_element,
        }
    }
}

pub mod matchers {
    pub mod general_mode {
        use std::iter::Peekable;

        use crate::lexer::{
            CnvToken, IssueManager, LexerInput, LexerIssue, LexerOutput, TokenizationSettings,
        };

        use super::configurable::*;

        pub fn match_resolvable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            issue_manager: &mut IssueManager<LexerIssue>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_resolvable_configurable(
                input,
                issue_manager,
                settings,
                &ResolvableMatcherConfig {
                    initial_predicate: is_part_of_resolvable,
                    loop_predicate: |_, c| is_part_of_resolvable(c),
                    error_loop_predicate: is_part_of_resolvable,
                },
            )
        }

        pub fn match_symbol(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            issue_manager: &mut IssueManager<LexerIssue>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_symbol_configurable(
                input,
                issue_manager,
                settings,
                &SymbolMatcherConfig {
                    symbol_mapper: try_map_symbol,
                },
            )
        }

        fn is_part_of_resolvable(c: char) -> bool {
            c.is_alphanumeric() || c == '_' || c == '.' || c == '-'
        }

        fn try_map_symbol(c: char) -> Option<CnvToken> {
            match c {
                '@' => Some(CnvToken::At),
                '^' => Some(CnvToken::Caret),
                ',' => Some(CnvToken::Comma),
                '!' => Some(CnvToken::Bang),
                ';' => Some(CnvToken::Semicolon),
                '(' => Some(CnvToken::LeftParenthesis),
                ')' => Some(CnvToken::RightParenthesis),
                '[' => Some(CnvToken::LeftBracket),
                ']' => Some(CnvToken::RightBracket),
                '{' => Some(CnvToken::LeftBrace),
                '}' => Some(CnvToken::RightBrace),
                _ => None,
            }
        }
    }

    pub mod operation_mode {
        use std::iter::Peekable;

        use crate::lexer::{
            CnvToken, IssueManager, LexerInput, LexerIssue, LexerOutput, TokenizationSettings,
        };

        use super::configurable::*;

        pub fn match_resolvable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            issue_manager: &mut IssueManager<LexerIssue>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_resolvable_configurable(
                input,
                issue_manager,
                settings,
                &ResolvableMatcherConfig {
                    initial_predicate: is_part_of_resolvable,
                    loop_predicate: |_, c| is_part_of_resolvable(c),
                    error_loop_predicate: is_part_of_resolvable,
                },
            )
        }

        pub fn match_symbol(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            issue_manager: &mut IssueManager<LexerIssue>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_symbol_configurable(
                input,
                issue_manager,
                settings,
                &SymbolMatcherConfig {
                    symbol_mapper: try_map_symbol,
                },
            )
        }

        fn is_part_of_resolvable(c: char) -> bool {
            c.is_alphanumeric() || c == '_' || c == '.'
        }

        fn try_map_symbol(c: char) -> Option<CnvToken> {
            match c {
                '+' => Some(CnvToken::Plus),
                '-' => Some(CnvToken::Minus),
                '*' => Some(CnvToken::Asterisk),
                '@' => Some(CnvToken::At),
                '%' => Some(CnvToken::Percent),
                '^' => Some(CnvToken::Caret),
                ',' => Some(CnvToken::Comma),
                '(' => Some(CnvToken::LeftParenthesis),
                ')' => Some(CnvToken::RightParenthesis),
                '[' => Some(CnvToken::LeftBracket),
                ']' => Some(CnvToken::RightBracket),
                _ => None,
            }
        }
    }

    mod configurable {
        use std::iter::Peekable;

        use crate::{
            common::{Bounds, Token, WithPosition},
            lexer::{
                CnvToken, IssueManager, LexerError, LexerFatal, LexerInput, LexerIssue,
                LexerOutput, TokenizationSettings,
            },
        };

        pub struct ResolvableMatcherConfig {
            pub initial_predicate: fn(char) -> bool,
            pub loop_predicate: fn(&str, char) -> bool,
            pub error_loop_predicate: fn(char) -> bool,
        }

        pub fn match_resolvable_configurable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            issue_manager: &mut IssueManager<LexerIssue>,
            settings: &TokenizationSettings,
            matcher_config: &ResolvableMatcherConfig,
        ) -> Option<LexerOutput> {
            if !matches!(input.peek(), Some(WithPosition { value: Ok(c), .. }) if (matcher_config.initial_predicate)(*c))
            {
                return None;
            }
            let start_position = input.peek().unwrap().position;
            let mut end_position = start_position;
            let mut lexeme = String::new();
            lexeme.push(input.next().unwrap().value.unwrap());
            while let Some(p) = input.next_if(|l| match &l.value {
                Ok(c) => (matcher_config.loop_predicate)(lexeme.as_ref(), *c),
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
                                        .is_ok_and(|c| (matcher_config.error_loop_predicate)(*c))
                                })
                                .is_some()
                            {
                                lexeme_end = input.next().map(|l| l.position).unwrap_or(lexeme_end);
                            }
                            issue_manager.emit_issue(LexerIssue::Error(
                                LexerError::LexemeTooLong {
                                    bounds: Bounds::new(start_position, lexeme_end),
                                    max_allowed_len: settings.max_lexeme_length,
                                },
                            ));
                            break;
                        }
                        lexeme.push(character);
                    }
                    Err(err) => {
                        issue_manager.emit_issue(LexerIssue::Fatal(LexerFatal::IoError {
                            position: p.position,
                            source: std::io::Error::from(err.kind()),
                        }));

                        return Some(Err(LexerFatal::IoError {
                            position: p.position,
                            source: err,
                        }));
                    }
                }
            }
            Some(Ok(Token {
                value: CnvToken::Resolvable(lexeme),
                bounds: Bounds {
                    start: start_position,
                    end: end_position,
                },
                had_errors: issue_manager.had_errors(),
            }))
        }

        pub struct SymbolMatcherConfig {
            pub symbol_mapper: fn(char) -> Option<CnvToken>,
        }

        pub fn match_symbol_configurable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            issue_manager: &mut IssueManager<LexerIssue>,
            _: &TokenizationSettings,
            matcher_config: &SymbolMatcherConfig,
        ) -> Option<LexerOutput> {
            let Some(WithPosition { value: Ok(c), .. }) = input.peek() else {
                return None;
            };
            if let Some(token) = (matcher_config.symbol_mapper)(*c) {
                let position = input.next().unwrap().position;
                Some(Ok(Token {
                    value: token,
                    bounds: Bounds {
                        start: position,
                        end: position,
                    },
                    had_errors: issue_manager.had_errors(),
                }))
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CnvToken {
    Unexpected(char),

    Resolvable(String),

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
