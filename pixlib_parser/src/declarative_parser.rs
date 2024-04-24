use thiserror::Error;

use crate::common::{Bounds, Issue, IssueKind, IssueManager, Position, Spanned};
use std::iter::Peekable;

type ParserInput = Spanned<char, Position, std::io::Error>;
type ParserOutput = Spanned<CnvDeclaration, Position, ParserFatal>;

#[derive(Debug)]
pub enum CnvDeclaration {
    ObjectInitialization(String),
    PropertyAssignment {
        parent: String,
        property: String,
        property_key: Option<String>,
        value: String,
    },
}

#[derive(Debug, Clone)]
pub struct ParsingSettings {
    max_line_length: usize,
}

impl Default for ParsingSettings {
    fn default() -> Self {
        Self {
            max_line_length: i32::MAX as usize,
        }
    }
}

#[derive(Error, Debug)]
pub enum ParserFatal {
    #[error("IO error at {position}")]
    IoError {
        position: Position,
        source: std::io::Error,
    },
}

#[derive(Error, Debug, Clone)]
pub enum ParserError {
    #[error("Expected character '{character}' at {position}")]
    ExpectedCharacter { position: Position, character: char },
    #[error("Expected keyword \"{keyword}\" at {position}")]
    ExpectedKeyword {
        position: Position,
        keyword: &'static str,
    },
    #[error("Unexpected character '{character}' at {position}")]
    UnexpectedCharacter { position: Position, character: char },
    #[error("Unexpected end of text at {position}")]
    UnexpectedEtx { position: Position },
    #[error("Lexeme length over limit ({max_allowed_len}) at {} to {}", bounds.start, bounds.end)]
    LineTooLong {
        bounds: Bounds,
        max_allowed_len: usize,
    },
}

impl From<ParserError> for ParserIssue {
    fn from(value: ParserError) -> Self {
        Self::Error(value)
    }
}

#[derive(Error, Debug, Clone)]
pub enum ParserWarning {}

#[derive(Error, Debug)]
pub enum ParserIssue {
    #[error("Fatal error: {0}")]
    Fatal(ParserFatal),
    #[error("Error: {0}")]
    Error(ParserError),
    #[error("Warning: {0}")]
    Warning(ParserWarning),
}

impl Issue for ParserIssue {
    fn kind(&self) -> IssueKind {
        match *self {
            Self::Fatal(_) => IssueKind::Fatal,
            Self::Error(_) => IssueKind::Error,
            Self::Warning(_) => IssueKind::Warning,
        }
    }
}

#[derive(Debug)]
pub struct DeclarativeParser<I: Iterator<Item = ParserInput>> {
    input: Peekable<I>,
    issue_manager: IssueManager<ParserIssue>,
    settings: ParsingSettings,
    next_position: Position,
}

impl<I: Iterator<Item = ParserInput> + 'static> DeclarativeParser<I> {
    pub fn new(
        input: I,
        settings: ParsingSettings,
        issue_manager: IssueManager<ParserIssue>,
    ) -> Self {
        Self {
            input: input.peekable(),
            issue_manager,
            settings,
            next_position: Position::default(),
        }
    }

    fn next_if_char(&mut self, f: impl FnOnce(char) -> bool) -> Option<ParserInput> {
        self.input
            .next_if(|result| result.as_ref().is_ok_and(|(_, c, _)| f(*c)))
    }

    fn skip_line(&mut self, mut had_slash: bool) {
        while let Some(result) = self.next_if_char(|c| c != '\n' || had_slash) {
            let (_, c, next_position) = result.unwrap();
            match c {
                '/' => had_slash = true,
                '\n' => had_slash = false,
                _ => (),
            }
            self.next_position = next_position;
        }
        if let Some(result) = self.next_if_char(|c| c == '\n') {
            self.next_position = result.unwrap().2
        }
    }
}

#[derive(Clone, Debug, Default)]
struct LineState {
    pub start_position: Option<Position>,
    pub next_position: Option<Position>,
    pub had_slash: bool,
    pub had_non_whitespace: bool,
    pub content: String,
}

impl LineState {
    pub fn reset(&mut self) {
        self.start_position = None;
        self.next_position = None;
        self.had_slash = false;
        self.had_non_whitespace = false;
        // println!("Resetting line state: {}", &self.content);
        self.content.clear();
    }

    pub fn into_line_to_split(self) -> LineToSplit {
        LineToSplit {
            content: self.content,
            start_position: self.start_position.unwrap_or_default(),
            next_position: self.next_position.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Default)]
struct LineToSplit {
    pub content: String,
    pub colon_index: Option<usize>,
    pub caret_index: Option<usize>,
    pub eq_index: Option<usize>,
    pub start_position: Position,
    pub next_position: Position,
}

impl LineToSplit {
    pub fn split(
        self,
        issue_manager: &mut IssueManager<ParserIssue>,
    ) -> (Position, CnvDeclaration, Position) {
        let declaration = if let Some(colon_index) = self.colon_index {
            let property = if let Some(caret_index) = self.caret_index {
                self.content[(colon_index + 1)..caret_index].to_owned()
            } else if let Some(eq_index) = self.eq_index {
                self.content[(colon_index + 1)..eq_index]
                    .trim_end()
                    .to_owned()
            } else {
                self.content[(colon_index + 1)..].to_owned()
            };
            let property_key = self.caret_index.map(|i| {
                if let Some(eq_index) = self.eq_index {
                    self.content[(i + 1)..eq_index].trim_end().to_owned()
                } else {
                    self.content[(i + 1)..].to_owned()
                }
            });
            let value = if let Some(eq_index) = self.eq_index {
                self.content[(eq_index + 1)..].trim_start().to_owned()
            } else {
                self.content[..0].to_owned()
            };
            let mut parent = self.content;
            parent.truncate(colon_index);
            CnvDeclaration::PropertyAssignment {
                parent,
                property,
                property_key,
                value,
            }
        } else if let Some(eq_index) = self.eq_index {
            // println!("##### \"{}\", \"{}\"", self.content[..eq_index].to_uppercase(), &self.content[6..eq_index]);
            let offset = if !(self.content[..eq_index]
                .to_uppercase()
                .starts_with("OBJECT")
                && self.content[6..eq_index].chars().all(|c| c.is_whitespace()))
            {
                issue_manager.emit_issue(
                    ParserError::ExpectedKeyword {
                        position: self.start_position,
                        keyword: "OBJECT",
                    }
                    .into(),
                );
                0
            } else {
                "OBJECT".len()
            } + 1;
            let mut name = self.content;
            let first_non_whitespace = &name[(eq_index + 1)..]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(eq_index);
            name.drain(..(first_non_whitespace + offset));
            CnvDeclaration::ObjectInitialization(name)
        } else {
            issue_manager.emit_issue(
                ParserError::ExpectedCharacter {
                    position: self.next_position,
                    character: '=',
                }
                .into(),
            );
            CnvDeclaration::ObjectInitialization(self.content)
        };
        (self.start_position, declaration, self.next_position)
    }
}

impl<I: Iterator<Item = ParserInput> + 'static> Iterator for DeclarativeParser<I> {
    type Item = ParserOutput;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.peek()?;
        let mut line_state = LineState::default();
        while let Some(result) = self
            .next_if_char(|c| c != '\n' || line_state.had_slash || !line_state.had_non_whitespace)
        {
            let (position, c, next_position) = result.unwrap();
            // println!("Current char: {}", c);
            line_state.start_position = line_state.start_position.or(Some(position));
            line_state.next_position = Some(self.next_position.assign(next_position));
            if !line_state.had_non_whitespace && !c.is_whitespace() && c != '/' {
                line_state.had_non_whitespace = true;
                if c == '#' {
                    self.skip_line(line_state.had_slash);
                    line_state.reset();
                    continue;
                }
            }
            match c {
                '/' => line_state.had_slash = true,
                '\n' if !line_state.had_slash && !line_state.had_non_whitespace => {
                    line_state.reset()
                }
                '\n' => line_state.had_slash = false,
                _ if !line_state.had_non_whitespace && c.is_whitespace() => (),
                _ => line_state.content.push(c),
            }
            if line_state.content.len() >= self.settings.max_line_length {
                self.skip_line(line_state.had_slash);
                self.issue_manager.emit_issue(
                    ParserError::LineTooLong {
                        bounds: Bounds::new(
                            line_state.start_position.unwrap_or_default(),
                            self.next_position,
                        ),
                        max_allowed_len: self.settings.max_line_length,
                    }
                    .into(),
                );
            }
        }
        if let Some(result) = self.next_if_char(|c| c == '\n') {
            let (_, _, next_position) = result.unwrap();
            line_state.next_position = Some(self.next_position.assign(next_position));
        }
        if self.input.peek().is_none() && line_state.content.is_empty() {
            return None;
        }
        let mut line_to_split = line_state.into_line_to_split();
        for (i, c) in line_to_split.content.chars().enumerate() {
            match c {
                '=' => {
                    if line_to_split.eq_index.is_some() {
                        let error = ParserError::UnexpectedCharacter {
                            position: &line_to_split.start_position + i,
                            character: c,
                        };
                        self.issue_manager.emit_issue(error.clone().into());
                        return Some(Ok(line_to_split.split(&mut self.issue_manager)));
                    } else {
                        line_to_split.eq_index = Some(i);
                    }
                }
                ':' if line_to_split.eq_index.is_none() => {
                    if line_to_split.colon_index.is_some() {
                        let error = ParserError::UnexpectedCharacter {
                            position: &line_to_split.start_position + i,
                            character: c,
                        };
                        self.issue_manager.emit_issue(error.clone().into());
                        return Some(Ok(line_to_split.split(&mut self.issue_manager)));
                    } else {
                        line_to_split.colon_index = Some(i);
                    }
                }
                '^' if line_to_split.eq_index.is_none() => {
                    if line_to_split.colon_index.is_none() || line_to_split.caret_index.is_some() {
                        let error = ParserError::UnexpectedCharacter {
                            position: &line_to_split.start_position + i,
                            character: c,
                        };
                        self.issue_manager.emit_issue(error.clone().into());
                        return Some(Ok(line_to_split.split(&mut self.issue_manager)));
                    } else {
                        line_to_split.caret_index = Some(i);
                    }
                }
                _ => (),
            }
        }
        Some(Ok(line_to_split.split(&mut self.issue_manager)))
    }
}
