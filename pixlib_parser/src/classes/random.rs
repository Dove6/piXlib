use std::any::Any;

use rand::Rng;

use crate::runner::RunnerError;

use super::*;

#[derive(Debug, Clone)]
pub struct RandomInit {
    // RAND
}

#[derive(Debug, Clone)]
pub struct Random {
    parent: Arc<CnvObject>,
    initial_properties: RandomInit,
}

impl Random {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: RandomInit) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn get(&self, max_exclusive: usize, offset: isize) -> RunnerResult<isize> {
        // GET
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(0..max_exclusive) as isize + offset)
    }

    pub fn get_plenty() {
        // GETPLENTY
        todo!()
    }
}

impl CnvType for Random {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "RANDOM"
    }

    fn has_event(&self, _name: &str) -> bool {
        false
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
        arguments: &[CnvValue],
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("GET") => match arguments.len() {
                0 => Err(RunnerError::TooFewArguments {
                    expected_min: 1,
                    actual: 0,
                }),
                1 => self.get(arguments[0].to_integer() as usize, 0),
                2 => self.get(
                    arguments[1].to_integer() as usize,
                    arguments[0].to_integer() as isize,
                ),
                arg_count => Err(RunnerError::TooManyArguments {
                    expected_max: 2,
                    actual: arg_count,
                }),
            }
            .map(|v| Some(CnvValue::Integer(v as i32))),
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        _properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        Ok(Self::from_initial_properties(parent, RandomInit {}))
    }
}
