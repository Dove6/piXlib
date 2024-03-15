use crate::common::{Bounds, Element, MultiModeLexer, PositionalIterator};

pub enum CnvTokenizationModes {
    General,
    Operation,
}

pub struct TokenizationSettings {
    max_lexeme_length: usize,
}

impl Default for TokenizationSettings {
    fn default() -> Self {
        Self { max_lexeme_length: i32::MAX as usize }
    }
}

pub struct CnvLexer<I: PositionalIterator<Item = char>> {
    input: I,
    settings: TokenizationSettings,
    tokenization_mode_stack: Vec<CnvTokenizationModes>,
    pub current_element: Element<CnvToken>,
}

type Matcher<I> = fn(&mut I, settings: &TokenizationSettings) -> std::io::Result<Option<Element<CnvToken>>>;

impl<I: PositionalIterator<Item = char> + 'static> CnvLexer<I> {
    const GENERAL_MATCHERS: &'static [Matcher<I>] =
        &[match_etx, match_resolvable, match_symbol,];
    const OPERATION_MATCHERS: &'static [Matcher<I>] = &[
        match_etx,
        match_operation_resolvable,
        match_operation_symbol,
    ];

    pub fn new(input: I, settings: TokenizationSettings) -> Self {
        Self {
            input,
            settings,
            tokenization_mode_stack: Vec::new(),
            current_element: Element::BeforeStream,
        }
    }
}

impl<I: PositionalIterator<Item = char>> MultiModeLexer for CnvLexer<I> {
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

impl<I: PositionalIterator<Item = char> + 'static> PositionalIterator for CnvLexer<I> {
    type Item = CnvToken;

    fn advance(&mut self) -> std::io::Result<crate::common::Element<Self::Item>> {
        if self.input.get_current_element() == &Element::BeforeStream {
            self.input.advance()?;
        }
        let new_element = if self.input.get_current_element() == &Element::AfterStream {
            Element::AfterStream
        } else {
            let matchers = match self.get_mode() {
                CnvTokenizationModes::General => &Self::GENERAL_MATCHERS,
                CnvTokenizationModes::Operation => &Self::OPERATION_MATCHERS,
            };
            let mut matched_element = None;
            for matcher in matchers.iter() {
                matched_element = matcher(&mut self.input, &self.settings)?;
                if matched_element.is_some() {
                    break;
                }
            }
            if let Some(matched_element) = matched_element {
                matched_element
            } else {
                let Element::WithinStream { element, bounds } = self.input.advance()? else {
                    panic!();
                };
                Element::WithinStream {
                    element: CnvToken::Unknown(element),
                    bounds,
                }
            }
        };
        Ok(std::mem::replace(&mut self.current_element, new_element))
    }

    fn get_current_element(&self) -> &crate::common::Element<Self::Item> {
        &self.current_element
    }
}

fn match_etx(
    input: &mut impl PositionalIterator<Item = char>,
    _: &TokenizationSettings,
) -> std::io::Result<Option<Element<CnvToken>>> {
    // println!("match etx");
    if input.get_current_element() == &Element::AfterStream {
        input.advance()?;
        Ok(Some(Element::AfterStream))
    } else {
        Ok(None)
    }
}

fn is_part_of_resolvable(c: &char) -> bool {
    c.is_alphanumeric() || *c == '_' || *c == '.' || *c == '-'
}

fn match_resolvable(
    input: &mut impl PositionalIterator<Item = char>,
    settings: &TokenizationSettings,
) -> std::io::Result<Option<Element<CnvToken>>> {
    let mut current_element = input.get_current_element().get_element();
    if current_element.is_none() || !current_element.is_some_and(is_part_of_resolvable) {
        return Ok(None);
    }
    let start_position = input.get_current_element().get_bounds().unwrap().start;
    let mut end_position = input.get_current_element().get_bounds().unwrap().end;
    let mut lexeme = String::new();
    while current_element.is_some_and(is_part_of_resolvable) {
        end_position = input.get_current_element().get_bounds().unwrap().end;
        if lexeme.len() >= settings.max_lexeme_length {
            return Err(std::io::Error::from(std::io::ErrorKind::Other));  // TODO: introduce own error type
        }
        lexeme.push(input.advance()?.unwrap());
        current_element = input.get_current_element().get_element();
    }
    Ok(Some(Element::WithinStream {
        element: CnvToken::Resolvable(lexeme),
        bounds: Bounds {
            start: start_position,
            end: end_position,
        },
    }))
}

fn match_symbol(
    input: &mut impl PositionalIterator<Item = char>,
    _: &TokenizationSettings,
) -> std::io::Result<Option<Element<CnvToken>>> {
    let token = match input.get_current_element().get_element() {
        Some('@') => CnvToken::At,
        Some('^') => CnvToken::Caret,
        Some(',') => CnvToken::Comma,
        Some('!') => CnvToken::Bang,
        Some(';') => CnvToken::Semicolon,
        Some('(') => CnvToken::LeftParenthesis,
        Some(')') => CnvToken::RightParenthesis,
        Some('[') => CnvToken::LeftBracket,
        Some(']') => CnvToken::RightBracket,
        Some('{') => CnvToken::LeftBrace,
        Some('}') => CnvToken::RightBrace,
        _ => return Ok(None),
    };
    let position = input.get_current_element().get_bounds().unwrap().start;
    input.advance()?;
    Ok(Some(Element::WithinStream {
        element: token,
        bounds: Bounds {
            start: position,
            end: position,
        },
    }))
}

fn is_part_of_operation_resolvable(c: &char) -> bool {
    c.is_alphanumeric() || *c == '_'
}

fn match_operation_resolvable(
    input: &mut impl PositionalIterator<Item = char>,
    settings: &TokenizationSettings,
) -> std::io::Result<Option<Element<CnvToken>>> {
    let mut current_element = input.get_current_element().get_element();
    if current_element.is_none() || !current_element.is_some_and(is_part_of_operation_resolvable) {
        return Ok(None);
    }
    let start_position = input.get_current_element().get_bounds().unwrap().start;
    let mut end_position = input.get_current_element().get_bounds().unwrap().end;
    let mut lexeme = String::new();
    while current_element.is_some_and(is_part_of_operation_resolvable) {
        end_position = input.get_current_element().get_bounds().unwrap().end;
        if lexeme.len() >= settings.max_lexeme_length {
            return Err(std::io::Error::from(std::io::ErrorKind::Other));  // TODO: introduce own error type
        }
        lexeme.push(input.advance()?.unwrap());
        current_element = input.get_current_element().get_element();
    }
    Ok(Some(Element::WithinStream {
        element: CnvToken::Resolvable(lexeme),
        bounds: Bounds {
            start: start_position,
            end: end_position,
        },
    }))
}

fn match_operation_symbol(
    input: &mut impl PositionalIterator<Item = char>,
    _: &TokenizationSettings,
) -> std::io::Result<Option<Element<CnvToken>>> {
    let token = match input.get_current_element().get_element() {
        Some('+') => CnvToken::Plus,
        Some('-') => CnvToken::Minus,
        Some('*') => CnvToken::Asterisk,
        Some('@') => CnvToken::At,
        Some('%') => CnvToken::Percent,
        Some('^') => CnvToken::Caret,
        Some(',') => CnvToken::Comma,
        Some('(') => CnvToken::LeftParenthesis,
        Some(')') => CnvToken::RightParenthesis,
        Some('[') => CnvToken::LeftBracket,
        Some(']') => CnvToken::RightBracket,
        _ => return Ok(None),
    };
    let position = input.get_current_element().get_bounds().unwrap().start;
    input.advance()?;
    Ok(Some(Element::WithinStream {
        element: token,
        bounds: Bounds {
            start: position,
            end: position,
        },
    }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CnvToken {
    EndOfText,
    Unknown(char),

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
