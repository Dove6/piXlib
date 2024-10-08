use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use lalrpop_util::ParseError;
use thiserror::Error;

use crate::{
    common::{DroppableRefMut, Issue, OkResult},
    parser::declarative_parser::ParserIssue,
    runner::{CnvScript, CnvValue, RunnerContext},
};

use super::{
    classes::{CnvTypeFactory, DummyCnvType},
    initable::Initable,
    parsers::{discard_if_empty, ProgramParsingError, TypeParsingError},
    CallableIdentifier, CnvContent,
};
use OkResult::{NoError, WithError};

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
    ) -> OkResult<&mut Self, ObjectBuilderError> {
        if let Some(old_value) = self.properties.insert(property.clone(), value) {
            return WithError(
                self,
                ObjectBuilderError::new(
                    self.name.clone(),
                    ObjectBuildErrorKind::PropertyAlreadyPresent(property, old_value),
                ),
            );
        };
        NoError(self)
    }

    pub fn build(self) -> Result<Arc<CnvObject>, ObjectBuilderError> {
        let mut properties = self.properties;
        let Some(type_name) = properties.remove("TYPE").and_then(discard_if_empty) else {
            return Err(ObjectBuilderError::new(
                self.name,
                ObjectBuildErrorKind::MissingType,
            )); // TODO: readable errors
        };
        let mut object = Arc::new(CnvObject {
            parent: self.parent,
            name: self.name.clone(),
            index: self.index,
            initialized: RwLock::new(false),
            content: CnvContent::None(DummyCnvType {}),
        });
        let content =
            CnvTypeFactory::create(Arc::clone(&object), type_name, properties).map_err(|e| {
                ObjectBuilderError::new(self.name, ObjectBuildErrorKind::ParsingError(e))
            })?;
        unsafe {
            Arc::get_mut_unchecked(&mut object).content = content;
        }
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
    pub initialized: RwLock<bool>,
    pub content: CnvContent,
}

impl PartialEq for CnvObject {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.index == other.index && self.name == other.name
    }
}

impl Eq for CnvObject {}

impl Hash for CnvObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.index.hash(state);
        self.name.hash(state);
    }
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
            .field("content", &self.content.get_type_id())
            .finish()
    }
}

impl CnvObject {
    pub fn call_method(
        self: &Arc<Self>,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: Option<RunnerContext>,
    ) -> anyhow::Result<CnvValue> {
        let context = context
            .map(|c| c.with_current_object(self.clone()))
            .unwrap_or(RunnerContext::new_minimal(&self.parent.runner, self));
        // log::trace!(
        //     "[1] Calling method: {:?} of: {:?} with context {} and arguments: {:?}",
        //     identifier, self.name, context, arguments
        // );
        let arguments = if matches!(identifier, CallableIdentifier::Method(_)) {
            arguments
                .iter()
                .map(|v| v.to_owned().resolve(context.clone()))
                .collect::<Vec<_>>()
        } else {
            arguments.to_owned()
        };

        self.content
            .call_method(identifier.clone(), &arguments, context.clone())
        // .inspect(|v| {
        //     log::trace!(
        //         "[2] Called method: {:?} of: {:?} with context {}, arguments: {:?} and result: {:?}",
        //         identifier, self.name, context, arguments, v
        //     )
        // })
        // .inspect_err(|e| {
        //     log::trace!(
        //         "[2] Called method: {:?} of: {:?} with context {}, arguments: {:?} and error: {}",
        //         identifier, self.name, context, arguments, e
        //     )
        // })
    }

    pub fn init(self: &Arc<Self>, context: Option<RunnerContext>) -> anyhow::Result<()> {
        let as_initable: Option<&dyn Initable> = (&self.content).into();
        let Some(initable) = as_initable else {
            self.initialized
                .write()
                .unwrap()
                .use_and_drop_mut(|i| **i = true);
            return Ok(());
        };
        let context = context
            .map(|c| c.with_current_object(self.clone()))
            .unwrap_or(RunnerContext::new_minimal(&self.parent.runner, self));
        initable.initialize(context).inspect(|_| {
            self.initialized
                .write()
                .unwrap()
                .use_and_drop_mut(|i| **i = true)
        })
    }
}
