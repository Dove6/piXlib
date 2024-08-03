use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use lalrpop_util::ParseError;
use thiserror::Error;

use crate::{
    common::Issue,
    runner::{CnvScript, CnvValue, RunnerContext},
};

use super::{
    parsers::{discard_if_empty, ProgramParsingError, TypeParsingError},
    CallableIdentifier, CnvType, CnvTypeFactory, PropertyValue, RunnerResult,
};

#[derive(Debug, Clone)]
pub struct CnvObjectBuilder {
    parent: Arc<CnvScript>,
    path: Arc<Path>,
    name: String,
    index: usize,
    properties: HashMap<String, String>,
}

impl CnvObjectBuilder {
    pub fn new(parent: Arc<CnvScript>, path: Arc<Path>, name: String, index: usize) -> Self {
        Self {
            parent,
            path,
            name,
            index,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, property: String, value: String) {
        self.properties.insert(property, value); // TODO: report duplicates
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
            content: RefCell::new(None),
        });
        let content =
            CnvTypeFactory::create(Arc::clone(&object), type_name, properties).map_err(|e| {
                ObjectBuilderError::new(self.name, ObjectBuildErrorKind::ParsingError(e))
            })?;
        object.content.replace(Some(content));
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
}

#[derive(Debug)]
pub struct CnvObject {
    pub parent: Arc<CnvScript>,
    pub name: String,
    pub index: usize,
    pub content: RefCell<Option<Box<dyn CnvType>>>,
}

impl CnvObject {
    pub fn call_method(
        &self,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        println!("Calling method: {:?} of: {:?}", identifier, self.name);
        self.content
            .borrow_mut()
            .as_mut()
            .unwrap()
            .call_method(identifier, arguments, context)
    }

    pub fn get_property(&self, name: &str) -> Option<PropertyValue> {
        println!("Getting property: {:?} of: {:?}", name, self.name);
        self.content.borrow().as_ref().unwrap().get_property(name)
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
