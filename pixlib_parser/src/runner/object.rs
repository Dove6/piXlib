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
    parser::declarative_parser::ParserIssue,
    runner::{CnvScript, CnvValue, RunnerContext},
};

use super::{
    classes::{CnvTypeFactory, DummyCnvType},
    initable::Initable,
    parsers::{discard_if_empty, ProgramParsingError, TypeParsingError},
    CallableIdentifier, CnvContent, RunnerResult,
};

#[derive(Debug, Clone)]
pub struct CnvObjectBuilder {
    parent: Arc<CnvScript>,
    name: String,
    index: usize,
    properties: HashMap<String, String>,
}

#[allow(clippy::arc_with_non_send_sync)]
impl CnvObjectBuilder {
    pub fn new(parent: Arc<CnvScript>, name: String, index: usize) -> Self {
        Self {
            parent,
            name,
            index,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(
        &mut self,
        property: String,
        value: String,
    ) -> Result<&mut Self, ObjectBuilderError> {
        if let Some(old_value) = self.properties.insert(property.clone(), value) {
            return Err(ObjectBuilderError::new(
                self.name.clone(),
                ObjectBuildErrorKind::PropertyAlreadyPresent(property, old_value),
            ));
        };
        Ok(self)
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
    #[error("Property {0} already present with value {1}")]
    PropertyAlreadyPresent(String, String),
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
        let context = context
            .map(|c| c.with_current_object(self.clone()))
            .unwrap_or(RunnerContext::new_minimal(&self.parent.runner, self));
        let arguments = if matches!(identifier, CallableIdentifier::Method(_)) {
            arguments
                .iter()
                .map(|v| v.to_owned().resolve(context.clone()))
                .collect::<Vec<_>>()
        } else {
            arguments.to_owned()
        };
        println!(
            "Calling method: {:?} of: {:?} with arguments: {:?}",
            identifier, self.name, arguments
        );
        self.content
            .borrow()
            .call_method(identifier, &arguments, context)
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
