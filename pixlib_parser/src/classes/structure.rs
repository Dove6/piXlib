use std::any::Any;

use parsers::{discard_if_empty, parse_program, STRUCT_FIELDS_REGEX};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct StructInit {
    // STRUCT
    pub fields: Option<Vec<(String, TypeName)>>,

    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Struct {
    parent: Arc<CnvObject>,
    initial_properties: StructInit,
}

impl Struct {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: StructInit) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn get_field() {
        // GETFIELD
        todo!()
    }

    pub fn set() {
        // SET
        todo!()
    }

    pub fn set_field() {
        // SETFIELD
        todo!()
    }
}

impl CnvType for Struct {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "STRUCT"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let fields = properties
            .remove("FIELDS")
            .and_then(discard_if_empty)
            .map(|s| {
                s.split(',')
                    .map(|f| {
                        let m = STRUCT_FIELDS_REGEX.captures(f).unwrap();
                        (m[1].to_owned(), m[2].to_owned())
                    })
                    .collect()
            });
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            StructInit {
                fields,
                on_done,
                on_init,
                on_signal,
            },
        ))
    }
}
