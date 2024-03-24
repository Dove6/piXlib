use crate::common::{Bounds, ErrorManager, Locatable, MultiModeLexer, Position, WithPosition};
use std::iter::Peekable;

type LexerInput = WithPosition<std::io::Result<char>>;
type LexerOutput = Locatable<CnvToken>;
type LexemeMatcher<I> =
    fn(&mut I, &mut ErrorManager<LexerError>, &TokenizationSettings) -> Option<LexerOutput>;

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter {
        position: Position,
        character: char,
    },
    LexemeTooLong {
        bounds: Bounds,
        max_allowed_len: usize,
    },
    /* fatal */ IoError {
        position: Position,
        error: std::io::Error,
    },
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
    error_manager: ErrorManager<LexerError>,
    settings: TokenizationSettings,
    tokenization_mode_stack: Vec<CnvTokenizationModes>,
}

impl<I: Iterator<Item = LexerInput> + 'static> CnvLexer<I> {
    const GENERAL_MATCHERS: &'static [LexemeMatcher<Peekable<I>>] = &[
        matchers::general_mode::match_resolvable,
        matchers::general_mode::match_symbol,
    ];
    const OPERATION_MATCHERS: &'static [LexemeMatcher<Peekable<I>>] = &[
        matchers::operation_mode::match_resolvable,
        matchers::operation_mode::match_symbol,
    ];

    pub fn new(input: I, settings: TokenizationSettings) -> Self {
        Self {
            input: input.peekable(),
            error_manager: ErrorManager::default(),
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
        if self.error_manager.encountered_fatal {
            return None;
        }
        match self
            .get_matchers()
            .iter()
            .find_map(|matcher| matcher(&mut self.input, &mut self.error_manager, &self.settings))
        {
            None => self.input.next().and_then(|p| match p.value {
                Ok(character) => {
                    self.error_manager
                        .emit_error(LexerError::UnexpectedCharacter {
                            position: p.position,
                            character,
                        });
                    Some(Locatable {
                        value: CnvToken::Unexpected(character),
                        bounds: Bounds::unit(p.position),
                    })
                }
                Err(err) => {
                    self.error_manager.emit_error(LexerError::IoError {
                        position: p.position,
                        error: err,
                    });
                    self.error_manager.encountered_fatal = true;
                    None
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
            CnvToken, ErrorManager, LexerError, LexerInput, LexerOutput, TokenizationSettings,
        };

        use super::configurable::*;

        pub fn match_resolvable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            error_manager: &mut ErrorManager<LexerError>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_resolvable_configurable(
                input,
                error_manager,
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
            error_manager: &mut ErrorManager<LexerError>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_symbol_configurable(
                input,
                error_manager,
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
            CnvToken, ErrorManager, LexerError, LexerInput, LexerOutput, TokenizationSettings,
        };

        use super::configurable::*;

        pub fn match_resolvable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            error_manager: &mut ErrorManager<LexerError>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_resolvable_configurable(
                input,
                error_manager,
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
            error_manager: &mut ErrorManager<LexerError>,
            settings: &TokenizationSettings,
        ) -> Option<LexerOutput> {
            match_symbol_configurable(
                input,
                error_manager,
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
            common::{Bounds, Locatable, WithPosition},
            lexer::{
                CnvToken, ErrorManager, LexerError, LexerInput, LexerOutput, TokenizationSettings,
            },
        };

        pub struct ResolvableMatcherConfig {
            pub initial_predicate: fn(char) -> bool,
            pub loop_predicate: fn(&str, char) -> bool,
            pub error_loop_predicate: fn(char) -> bool,
        }

        pub fn match_resolvable_configurable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            error_manager: &mut ErrorManager<LexerError>,
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
                            error_manager.emit_error(LexerError::LexemeTooLong {
                                bounds: Bounds::new(start_position, lexeme_end),
                                max_allowed_len: settings.max_lexeme_length,
                            });
                            break;
                        }
                        lexeme.push(character);
                    }
                    Err(err) => {
                        error_manager.emit_error(LexerError::IoError {
                            position: p.position,
                            error: err,
                        });
                        error_manager.encountered_fatal = true;
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

        pub struct SymbolMatcherConfig {
            pub symbol_mapper: fn(char) -> Option<CnvToken>,
        }

        pub fn match_symbol_configurable(
            input: &mut Peekable<impl Iterator<Item = LexerInput>>,
            _: &mut ErrorManager<LexerError>,
            _: &TokenizationSettings,
            matcher_config: &SymbolMatcherConfig,
        ) -> Option<LexerOutput> {
            let Some(WithPosition { value: Ok(c), .. }) = input.peek() else {
                return None;
            };
            if let Some(token) = (matcher_config.symbol_mapper)(*c) {
                let position = input.next().unwrap().position;
                Some(Locatable {
                    value: token,
                    bounds: Bounds {
                        start: position,
                        end: position,
                    },
                })
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
