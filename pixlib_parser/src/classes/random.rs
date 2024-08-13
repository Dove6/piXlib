use std::{any::Any, cell::RefCell};

use content::EventHandler;
use rand::Rng;

use crate::{ast::ParsedScript, runner::RunnerError};

use super::*;

#[derive(Debug, Clone)]
pub struct RandProperties {
    // RAND
}

#[derive(Debug, Clone, Default)]
struct RandState {}

#[derive(Debug, Clone)]
pub struct RandEventHandlers {}

impl EventHandler for RandEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Rand {
    parent: Arc<CnvObject>,

    state: RefCell<RandState>,
    event_handlers: RandEventHandlers,
}

impl Rand {
    pub fn from_initial_properties(parent: Arc<CnvObject>, _props: RandProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(RandState {
                ..Default::default()
            }),
            event_handlers: RandEventHandlers {},
        }
    }
}

impl CnvType for Rand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "RANDOM"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("GET") => match arguments.len() {
                0 => Err(RunnerError::TooFewArguments {
                    expected_min: 1,
                    actual: 0,
                }),
                1 => self
                    .state
                    .borrow()
                    .get(arguments[0].to_integer() as usize, 0),
                2 => self.state.borrow().get(
                    arguments[1].to_integer() as usize,
                    arguments[0].to_integer() as isize,
                ),
                arg_count => Err(RunnerError::TooManyArguments {
                    expected_max: 2,
                    actual: arg_count,
                }),
            }
            .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPLENTY") => {
                self.state.borrow().get_plenty().map(|_| None)
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self.event_handlers.get(
                    event_name,
                    arguments.get(0).map(|v| v.to_string()).as_deref(),
                ) {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        _properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        Ok(CnvContent::Rand(Rand::from_initial_properties(
            parent,
            RandProperties {},
        )))
    }
}

impl RandState {
    pub fn get(&self, max_exclusive: usize, offset: isize) -> RunnerResult<isize> {
        // GET
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(0..max_exclusive) as isize + offset)
    }

    pub fn get_plenty(&self) -> RunnerResult<()> {
        // GETPLENTY
        todo!()
    }
}
