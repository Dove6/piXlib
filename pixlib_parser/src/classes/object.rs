use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use lalrpop_util::ParseError;
use thiserror::Error;

use crate::{
    common::{DroppableRefMut, Issue},
    declarative_parser::ParserIssue,
    runner::{CnvScript, CnvValue, RunnerContext},
};

use super::{
    initable::Initable,
    parsers::{discard_if_empty, ProgramParsingError, TypeParsingError},
    CallableIdentifier, CnvContent, CnvTypeFactory, DummyCnvType, PropertyValue, RunnerResult,
};

#[derive(Debug, Clone)]
pub struct CnvObjectBuilder {
    parent: Arc<CnvScript>,
    name: String,
    index: usize,
    properties: HashMap<String, String>,
}

impl CnvObjectBuilder {
    pub fn new(parent: Arc<CnvScript>, name: String, index: usize) -> Self {
        Self {
            parent,
            name,
            index,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, property: String, value: String) -> &mut Self {
        self.properties.insert(property, value); // TODO: report duplicates
        self
    }

    pub fn build(self) -> Result<Arc<CnvObject>, ObjectBuilderError> {
        let mut properties = self.properties;
        let Some(type_name) = properties.remove("TYPE").and_then(discard_if_empty) else {
            return Err(ObjectBuilderError::new(
                self.name,
                ObjectBuildErrorKind::MissingType,
            )); // TODO: readable errors
        };
        let object = Arc::new(CnvObject {
            parent: self.parent,
            name: self.name.clone(),
            index: self.index,
            initialized: RefCell::new(false),
            content: RefCell::new(CnvContent::None(DummyCnvType {})),
        });
        let content =
            CnvTypeFactory::create(Arc::clone(&object), type_name, properties).map_err(|e| {
                ObjectBuilderError::new(self.name, ObjectBuildErrorKind::ParsingError(e))
            })?;
        object.content.replace(content);
        Ok(object)
    }
}

#[derive(Debug, Error)]
#[error("Error building object {name}: {source}")]
pub struct ObjectBuilderError {
    pub name: String,
    pub path: Arc<Path>,
    pub source: Box<ObjectBuildErrorKind>,
}

impl ObjectBuilderError {
    pub fn new(name: String, source: ObjectBuildErrorKind) -> Self {
        Self {
            name,
            path: PathBuf::from(".").into(),
            source: Box::new(source),
        }
    }
}

impl Issue for ObjectBuilderError {
    fn kind(&self) -> crate::common::IssueKind {
        match *self.source {
            ObjectBuildErrorKind::ParsingError(TypeParsingError::InvalidProgram(
                ProgramParsingError(ParseError::User { .. }),
            )) => crate::common::IssueKind::Fatal,
            _ => crate::common::IssueKind::Fatal,
        }
    }
}

#[derive(Debug, Error)]
pub enum ObjectBuildErrorKind {
    #[error("Missing type property")]
    MissingType,
    #[error("Parsing error: {0}")]
    ParsingError(TypeParsingError),
    #[error("Parser issue: {0}")]
    ParserIssue(ParserIssue),
}

pub struct CnvObject {
    pub parent: Arc<CnvScript>,
    pub name: String,
    pub index: usize,
    pub initialized: RefCell<bool>,
    pub content: RefCell<CnvContent>,
}

impl core::fmt::Debug for CnvObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CnvObject")
            .field(
                "parent",
                &format!(
                    "CnvScript with {} objects",
                    &self.parent.objects.borrow().len()
                ),
            )
            .field("name", &self.name)
            .field("index", &self.index)
            .field("content", &self.content.borrow().get_type_id())
            .finish()
    }
}

impl CnvObject {
    pub fn call_method(
        self: &Arc<Self>,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: Option<RunnerContext>,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of: {:?}", identifier, self.name);
        let context = context
            .map(|c| c.with_current_object(self.clone()))
            .unwrap_or(RunnerContext::new_minimal(&self.parent.runner, self));
        self.content
            .borrow()
            .call_method(identifier, arguments, context)
        // println!("Result is {:?}", result);
    }

    pub fn init(self: &Arc<Self>, context: Option<RunnerContext>) -> RunnerResult<()> {
        let mut content = self.content.borrow_mut();
        let as_initable: Option<&mut dyn Initable> = (&mut *content).into();
        let Some(initable) = as_initable else {
            *self.initialized.borrow_mut() = true;
            return Ok(());
        };
        let context = context
            .map(|c| c.with_current_object(self.clone()))
            .unwrap_or(RunnerContext::new_minimal(&self.parent.runner, self));
        initable.initialize(context).inspect(|_| {
            self.initialized
                .borrow_mut()
                .use_and_drop_mut(|i| **i = true)
        })
    }
}

#[derive(Debug)]
pub enum MemberInfo<'a> {
    Property(PropertyInfo<'a>),
    Callable(CallableInfo<'a>),
}

#[derive(Debug)]
pub struct PropertyInfo<'a> {
    name: &'a str,
    r#type: PropertyValue,
}

#[derive(Debug)]
pub struct CallableInfo<'a> {
    identifier: CallableIdentifier<'a>,
    parameters: &'a [PropertyInfo<'a>],
}
