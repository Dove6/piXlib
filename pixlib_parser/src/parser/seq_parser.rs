use thiserror::Error;

use crate::{
    common::{Bounds, IssueManager, Position, RemoveSearchable, Spanned},
    runner::RunnerError,
};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
    iter::Peekable,
    sync::Arc,
};

use super::declarative_parser::{ParserError, ParserFatal, ParserIssue};

pub type ParserInput = Spanned<char, Position, std::io::Error>;
type ParserOutput = Spanned<SeqDeclaration, Position, ParserFatal>;

#[derive(Debug, Clone)]
pub struct SeqBuilder {
    pub root_name: String,
}

#[derive(Debug, Clone, Error)]
pub enum SeqParserError {
    #[error("Sequence {0} not found in SEQ file")]
    ObjectNotFound(String),
    #[error("Leftover sequences: {0}")]
    LeftoverSequences(DisplayableVec<String>),
    #[error("Name missing for sequence {0}")]
    MissingNameDeclaration(String),
    #[error("Type missing for sequence {0}")]
    MissingTypeDeclaration(String),
    #[error("Mode missing for sequence {0}")]
    MissingModeDeclaration(String),
    #[error("Parameter name missing for SEQEVENT of sequence {0}")]
    MissingParameterName(String),
    #[error("Invalid parameter index {index} for sequence {name}")]
    InvalidParameterIndex { name: String, index: String },
    #[error("Invalid type {type_name} for sequence {name}")]
    InvalidSequenceType { name: String, type_name: String },
    #[error("Invalid mode {mode} for sequence {name}")]
    InvalidSequenceMode { name: String, mode: String },
    #[error("Invalid boolean value {value} for sequence {name}")]
    InvalidBooleanValue { name: String, value: String },
    #[error("Leftover declarations for sequence {name}: {declarations}")]
    LeftoverDeclarations {
        name: String,
        declarations: DisplayableVec<SeqDeclaration>,
    },
}

#[derive(Debug, Clone)]
pub struct DisplayableVec<T>(pub Vec<T>);

impl<T> DisplayableVec<T> {
    pub fn new(content: Vec<T>) -> Self {
        Self(content)
    }
}

impl<T: Display> Display for DisplayableVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        if !self.0.is_empty() {
            f.write_str(&self.0[0].to_string())?;
            for element in self.0.iter().skip(1) {
                f.write_str(", ")?;
                f.write_str(&element.to_string())?;
            }
        }
        f.write_char(']')
    }
}

impl From<SeqParserError> for RunnerError {
    fn from(value: SeqParserError) -> Self {
        Self::SeqParserError(value)
    }
}

impl SeqBuilder {
    pub fn new(root_name: String) -> Self {
        Self { root_name }
    }

    pub fn build<I: Iterator<Item = ParserOutput>>(
        self,
        input: I,
    ) -> anyhow::Result<Arc<SeqEntry>> {
        let mut cache: HashMap<String, Vec<SeqDeclaration>> = HashMap::new();
        for next in input {
            let (_, declaration, _) = next.map_err(RunnerError::ParserError)?;
            let name = match &declaration {
                SeqDeclaration::SequenceInitialization(name) => name.clone(),
                SeqDeclaration::PropertyAssignment { name, .. } => name.clone(),
                SeqDeclaration::NestingRequest { parent, .. } => parent.clone(),
            }
            .trim()
            .to_owned();
            cache
                .entry(name)
                .and_modify(|v| v.push(declaration.clone()))
                .or_insert(vec![declaration]);
        }
        let result = Self::build_entry(self.root_name, &mut cache)?;
        if !cache.is_empty() {
            Err(
                SeqParserError::LeftoverSequences(DisplayableVec(cache.into_keys().collect()))
                    .into(),
            )
        } else {
            Ok(result)
        }
    }

    fn build_entry(
        name: String,
        cache: &mut HashMap<String, Vec<SeqDeclaration>>,
    ) -> anyhow::Result<Arc<SeqEntry>> {
        let Some(mut dec_list) = cache.remove(&name) else {
            return Err(SeqParserError::ObjectNotFound(name).into());
        };
        if dec_list
            .remove_found(|d| matches!(d, SeqDeclaration::SequenceInitialization(_)))
            .is_none()
        {
            return Err(SeqParserError::MissingNameDeclaration(name).into());
        };
        let Some(SeqDeclaration::PropertyAssignment {
            value: type_name, .. // TODO: check if property key is empty
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("TYPE"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        match type_name.to_uppercase().as_ref() {
            "SIMPLE" => Self::build_simple_type(name, dec_list),
            "SPEAKING" => Self::build_speaking_type(name, dec_list),
            "SEQUENCE" => Self::build_sequence_type(name, dec_list, cache),
            other => Err(SeqParserError::InvalidSequenceType {
                name,
                type_name: other.to_owned(),
            }
            .into()),
        }
    }

    fn build_simple_type(
        name: String,
        mut dec_list: Vec<SeqDeclaration>,
    ) -> anyhow::Result<Arc<SeqEntry>> {
        let Some(SeqDeclaration::PropertyAssignment {
            value: filename, ..
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("FILENAME"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        let Some(SeqDeclaration::PropertyAssignment {
            value: event, ..
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("EVENT"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        if !dec_list.is_empty() {
            Err(SeqParserError::LeftoverDeclarations {
                name,
                declarations: DisplayableVec(dec_list),
            }
            .into())
        } else {
            Ok(Arc::new(SeqEntry {
                name,
                r#type: SeqType::Simple {
                    filename: filename.trim().to_ascii_uppercase(),
                    event: event.trim().to_ascii_uppercase(),
                },
            }))
        }
    }

    fn build_speaking_type(
        name: String,
        mut dec_list: Vec<SeqDeclaration>,
    ) -> anyhow::Result<Arc<SeqEntry>> {
        let Some(SeqDeclaration::PropertyAssignment {
            value: animation_filename, ..
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("ANIMOFN"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        let Some(SeqDeclaration::PropertyAssignment {
            value: sound_filename, ..
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("WAVFN"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        let Some(SeqDeclaration::PropertyAssignment {
            value: prefix, ..
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("PREFIX"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        let Some(SeqDeclaration::PropertyAssignment {
            value: starting, ..
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("STARTING"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        let starting = match starting.trim().to_uppercase().as_ref() {
            "TRUE" => true,
            "FALSE" => false,
            other => {
                return Err(SeqParserError::InvalidBooleanValue {
                    name,
                    value: other.to_owned(),
                }
                .into())
            }
        };
        let Some(SeqDeclaration::PropertyAssignment {
            value: ending, ..
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("ENDING"))) else {
            return Err(SeqParserError::MissingTypeDeclaration(name).into());
        };
        let ending = match ending.trim().to_uppercase().as_ref() {
            "TRUE" => true,
            "FALSE" => false,
            other => {
                return Err(SeqParserError::InvalidBooleanValue {
                    name,
                    value: other.to_owned(),
                }
                .into())
            }
        };
        if !dec_list.is_empty() {
            Err(SeqParserError::LeftoverDeclarations {
                name,
                declarations: DisplayableVec(dec_list),
            }
            .into())
        } else {
            Ok(Arc::new(SeqEntry {
                name,
                r#type: SeqType::Speaking {
                    animation_filename: animation_filename.trim().to_uppercase(),
                    sound_filename: sound_filename.trim().to_uppercase(),
                    prefix: prefix.trim().to_uppercase(),
                    starting,
                    ending,
                },
            }))
        }
    }

    fn build_sequence_type(
        name: String,
        mut dec_list: Vec<SeqDeclaration>,
        cache: &mut HashMap<String, Vec<SeqDeclaration>>,
    ) -> anyhow::Result<Arc<SeqEntry>> {
        let Some(SeqDeclaration::PropertyAssignment {
            value: mode, .. // TODO: check if property key is empty
        }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("MODE"))) else {
            return Err(SeqParserError::MissingModeDeclaration(name).into());
        };
        let mode = match mode.to_uppercase().as_ref() {
            "PARAMETER" => {
                let mut parameters = HashMap::new();
                while let Some(SeqDeclaration::PropertyAssignment { property_key, value, .. }) = dec_list.remove_found(|d| matches!(d, SeqDeclaration::PropertyAssignment { property, .. } if property.eq_ignore_ascii_case("SEQEVENT"))) {
                    let Some(parameter) = property_key else {return Err(SeqParserError::MissingParameterName(name.clone()).into())};
                    let index = Self::parse_index(&value).ok_or(SeqParserError::InvalidParameterIndex { name: name.clone(), index: value })?;
                    parameters.insert(parameter, index);
                }
                SeqMode::Parameter(parameters)
            }
            "RANDOM" => SeqMode::Random,
            "SEQUENCE" => SeqMode::Sequence,
            other => {
                return Err(SeqParserError::InvalidSequenceMode {
                    name,
                    mode: other.to_owned(),
                }
                .into())
            }
        };
        let mut children = Vec::new();
        while let Some(SeqDeclaration::NestingRequest { child, .. }) =
            dec_list.remove_found(|d| matches!(d, SeqDeclaration::NestingRequest { .. }))
        {
            children.push(Self::build_entry(child.trim().to_uppercase(), cache)?);
        }
        if !dec_list.is_empty() {
            Err(SeqParserError::LeftoverDeclarations {
                name,
                declarations: DisplayableVec(dec_list),
            }
            .into())
        } else {
            Ok(Arc::new(SeqEntry {
                name,
                r#type: SeqType::Sequence { mode, children },
            }))
        }
    }

    fn parse_index(value: &str) -> Option<usize> {
        let value = value.trim().bytes().next()?;
        Some(match value {
            b'1'..=b'9' => value - b'1',
            b';'..=b'Z' if value != b'=' => value - b'8',
            b'['..=b'~' => value - b'>',
            _ => return None,
        } as usize)
    }
}

#[derive(Debug, Clone)]
pub enum SeqDeclaration {
    SequenceInitialization(String),
    PropertyAssignment {
        name: String,
        property: String,
        property_key: Option<String>,
        value: String,
    },
    NestingRequest {
        parent: String,
        child: String,
    },
}

impl Display for SeqDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SeqDeclaration::")?;
        match self {
            SeqDeclaration::SequenceInitialization(name) => f.write_fmt(format_args!("SequenceInitialization({name})")),
            SeqDeclaration::PropertyAssignment { name, property, property_key, value } => f.write_fmt(format_args!("PropertyAssignment {{ name: {name}, property: {property}, property_key: {}, value: {value} }}", property_key.as_ref().map(|v| format!("Some({v})")).unwrap_or("None".to_owned()))),
            SeqDeclaration::NestingRequest { parent, child } => f.write_fmt(format_args!("NestingRequest {{ parent: {parent}, child: {child} }}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeqEntry {
    pub name: String,
    pub r#type: SeqType,
}

#[derive(Debug, Clone)]
pub enum SeqType {
    Simple {
        filename: String,
        event: String,
    },
    Speaking {
        animation_filename: String,
        sound_filename: String,
        prefix: String,
        starting: bool,
        ending: bool,
    },
    Sequence {
        mode: SeqMode,
        children: Vec<Arc<SeqEntry>>,
    },
}

#[derive(Debug, Clone)]
pub enum SeqMode {
    Parameter(HashMap<String, usize>), // matching root name
    Random,
    Sequence,
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

#[derive(Debug)]
pub struct SeqParser<I: Iterator<Item = ParserInput>> {
    input: Peekable<I>,
    issue_manager: IssueManager<ParserIssue>,
    settings: ParsingSettings,
    next_position: Position,
}

impl<I: Iterator<Item = ParserInput>> SeqParser<I> {
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

    fn skip_line(&mut self) {
        while let Some(result) = self.next_if_char(|c| c != '\n') {
            self.next_position = result.unwrap().2;
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
    pub had_non_whitespace: bool,
    pub content: String,
}

impl LineState {
    pub fn reset(&mut self) {
        self.start_position = None;
        self.next_position = None;
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
    pub second_colon_index: Option<usize>,
    pub eq_index: Option<usize>,
    pub eq_space_index: Option<usize>,
    pub start_position: Position,
    pub next_position: Position,
}

impl LineToSplit {
    pub fn split(
        self,
        issue_manager: &mut IssueManager<ParserIssue>,
    ) -> (Position, SeqDeclaration, Position) {
        let declaration = if let Some(colon_index) = self.colon_index {
            let property = if let Some(second_colon_index) = self.second_colon_index {
                self.content[(colon_index + 1)..second_colon_index].to_owned()
            } else if let Some(eq_index) = self.eq_index {
                self.content[(colon_index + 1)..eq_index]
                    .trim_end()
                    .to_owned()
            } else if let Some(eq_space_index) = self.eq_space_index {
                self.content[(colon_index + 1)..eq_space_index]
                    .trim_end()
                    .to_owned()
            } else {
                self.content[(colon_index + 1)..].to_owned()
            };
            let property_key = self.second_colon_index.map(|i| {
                if let Some(eq_index) = self.eq_index {
                    self.content[(i + 1)..eq_index].trim_end().to_owned()
                } else {
                    self.content[(i + 1)..].to_owned()
                }
            });
            let value = if let Some(eq_index) = self.eq_index {
                self.content[(eq_index + 1)..].trim_start().to_owned()
            } else if let Some(eq_space_index) = self.eq_space_index {
                self.content[(eq_space_index + 1)..].trim_start().to_owned()
            } else {
                self.content[..0].to_owned()
            };
            let mut name = self.content;
            name.truncate(colon_index);
            if property == "ADD" && self.eq_space_index.is_some() {
                SeqDeclaration::NestingRequest {
                    parent: value,
                    child: name,
                }
            } else {
                SeqDeclaration::PropertyAssignment {
                    name,
                    property,
                    property_key,
                    value,
                }
            }
        } else if let Some(eq_index) = self.eq_index {
            // println!("##### \"{}\", \"{}\"", self.content[..eq_index].to_uppercase(), &self.content[4..eq_index]);
            let offset = if !(self.content[..eq_index].to_uppercase().starts_with("NAME")  // TODO: strip_prefix
                && self.content[4..eq_index].chars().all(|c| c.is_whitespace()))
            {
                issue_manager.emit_issue(
                    ParserError::ExpectedKeyword {
                        position: self.start_position,
                        keyword: "NAME",
                    }
                    .into(),
                );
                0
            } else {
                "NAME".len()
            } + 1;
            let mut name = self.content;
            let first_non_whitespace = &name[(eq_index + 1)..]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(eq_index);
            name.drain(..(first_non_whitespace + offset));
            name.drain(
                ..name
                    .chars()
                    .position(|c| !c.is_whitespace())
                    .unwrap_or_default(),
            );
            SeqDeclaration::SequenceInitialization(name)
        } else {
            issue_manager.emit_issue(
                ParserError::ExpectedCharacter {
                    position: self.next_position,
                    character: '=',
                }
                .into(),
            );
            SeqDeclaration::SequenceInitialization(self.content)
        };
        (self.start_position, declaration, self.next_position)
    }
}

impl<I: Iterator<Item = ParserInput>> Iterator for SeqParser<I> {
    type Item = ParserOutput;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.peek()?;
        let mut line_state = LineState::default();
        while let Some(result) = self.next_if_char(|c| c != '\n' || !line_state.had_non_whitespace)
        {
            let (position, c, next_position) = result.unwrap();
            // println!("Current char: {}", c);
            line_state.start_position = line_state.start_position.or(Some(position));
            line_state.next_position = Some(self.next_position.assign(next_position));
            if !line_state.had_non_whitespace && !c.is_whitespace() {
                line_state.had_non_whitespace = true;
            }
            match c {
                '\n' if !line_state.had_non_whitespace => line_state.reset(),
                _ if !line_state.had_non_whitespace && c.is_whitespace() => {}
                '\r' => {}
                _ => line_state.content.push(c),
            }
            if line_state.content.len() >= self.settings.max_line_length {
                self.skip_line();
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
            line_state.next_position = Some(self.next_position.assign(result.unwrap().2));
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
                    if line_to_split.second_colon_index.is_some() {
                        let error = ParserError::UnexpectedCharacter {
                            position: &line_to_split.start_position + i,
                            character: c,
                        };
                        self.issue_manager.emit_issue(error.clone().into());
                        return Some(Ok(line_to_split.split(&mut self.issue_manager)));
                    } else if line_to_split.colon_index.is_some() {
                        line_to_split.second_colon_index = Some(i);
                    } else {
                        line_to_split.colon_index = Some(i);
                    }
                }
                ' ' if line_to_split.colon_index.is_some()
                    && line_to_split.eq_space_index.is_none() =>
                {
                    line_to_split.eq_space_index = Some(i);
                }
                _ => (),
            }
        }
        Some(Ok(line_to_split.split(&mut self.issue_manager)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::{Issue, IssueHandler, IssueManager, Position},
        runner::ObjectBuilderError,
    };
    use test_case::test_case;

    #[test]
    fn test_seq_parser() {
        let input = r"
NAME = MYSEQ
MYSEQ:TYPE = SEQUENCE
MYSEQ:MODE = PARAMETER
MYSEQ:SEQEVENT:CHILDSEQ = 1

NAME = CHILDSEQ
CHILDSEQ:TYPE = SEQUENCE
CHILDSEQ:MODE = RANDOM
CHILDSEQ:ADD MYSEQ
        ";

        let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
        parser_issue_manager.set_handler(Box::new(IssuePrinter));
        let mut issue_manager: IssueManager<ObjectBuilderError> = Default::default();
        issue_manager.set_handler(Box::new(IssuePrinter));
        let seq_parser = SeqParser::new(
            input.char_indices().map(|(i, c)| {
                Ok((
                    Position {
                        line: 1,
                        column: 1 + i,
                        character: i,
                    },
                    c,
                    Position {
                        line: 1,
                        column: 2 + i,
                        character: i + 1,
                    },
                ))
            }),
            Default::default(),
            parser_issue_manager,
        )
        .peekable();
        let builder = SeqBuilder::new("MYSEQ".to_owned());
        match builder.build(seq_parser) {
            Err(err) => panic!("{:?}", err),
            Ok(result) => println!("{:#?}", result),
        }
    }

    #[test_case("empty", "", None)]
    #[test_case("space", " ", None)]
    #[test_case("zero", "0", None)]
    #[test_case("one", "1", Some(0))]
    #[test_case("one with whitespace on left", " 1", Some(0))]
    #[test_case("one with whitespace on right", "1 ", Some(0))]
    #[test_case("one with whitespace on both sides", " 1 ", Some(0))]
    #[test_case("semicolon", ";", Some(3))]
    #[test_case("nine", "9", Some(8))]
    #[test_case("at", "@", Some(8))]
    #[test_case("capital A", "A", Some(9))]
    #[test_case("left bracket", "[", Some(29))]
    #[test_case("capital Z", "Z", Some(34))]
    #[test_case("backtick", "`", Some(34))]
    #[test_case("small a", "a", Some(35))]
    #[test_case("small z", "z", Some(60))]
    #[test_case("left brace", "{", Some(61))]
    #[test_case("pipe", "|", Some(62))]
    #[test_case("right brace", "}", Some(63))]
    #[test_case("tilde", "~", Some(64))]
    fn test_seqevent_index_parser(_description: &str, value: &str, expected: Option<usize>) {
        assert_eq!(SeqBuilder::parse_index(value), expected);
    }

    #[derive(Debug)]
    struct IssuePrinter;

    impl<I: Issue> IssueHandler<I> for IssuePrinter {
        fn handle(&mut self, issue: I) {
            eprintln!("{:?}", issue);
        }
    }
}
