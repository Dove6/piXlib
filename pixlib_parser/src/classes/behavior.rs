use std::{any::Any, cell::RefCell};

use initable::Initable;
use parsers::{discard_if_empty, parse_program};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

use super::*;

#[derive(Debug, Clone)]
pub struct BehaviorProperties {
    // BEHAVIOUR
    pub code: Option<Arc<ParsedScript>>,  // CODE
    pub condition: Option<ConditionName>, // CONDITION

    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct BehaviorState {
    // deduced from methods
    pub is_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct BehaviorEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Behavior {
    // BEHAVIOUR
    parent: Arc<CnvObject>,

    state: RefCell<BehaviorState>,
    event_handlers: BehaviorEventHandlers,

    code: Option<Arc<ParsedScript>>,
    condition: Option<ConditionName>,
}

impl Behavior {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: BehaviorProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(BehaviorState {
                is_enabled: true,
                ..Default::default()
            }),
            event_handlers: BehaviorEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
            },
            code: props.code,
            condition: props.condition,
        }
    }

    pub fn run(&self, context: RunnerContext) -> RunnerResult<()> {
        self.state.borrow().run(self, context)
    }
}

impl CnvType for Behavior {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BEHAVIOUR"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        let context = context.with_current_object(self.parent.clone());
        match name {
            CallableIdentifier::Method("BREAK") => self.state.borrow().break_run().map(|_| None),
            CallableIdentifier::Method("DISABLE") => {
                self.state.borrow_mut().disable().map(|_| None)
            }
            CallableIdentifier::Method("RUN") => {
                self.state.borrow().run(self, context).map(|_| None)
            }
            CallableIdentifier::Method("RUNC") => self.state.borrow().run_c().map(|_| None),
            CallableIdentifier::Method("RUNLOOPED") => {
                self.state.borrow().run_looped().map(|_| None)
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.event_handlers.on_done.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.event_handlers.on_init.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONSIGNAL") => {
                if let Some(v) = self
                    .event_handlers
                    .on_signal
                    .get(&arguments[0].to_string())
                    .as_ref()
                {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let code = properties
            .remove("CODE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let condition = properties.remove("CONDITION").and_then(discard_if_empty);
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
        let mut on_signal = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSIGNAL" {
                on_signal.insert(String::from(""), parse_program(v.to_owned())?);
            } else if k.starts_with("ONSIGNAL^") {
                on_signal.insert(String::from(&k[9..]), parse_program(v.to_owned())?);
            }
        }
        properties.retain(|k, _| k != "ONSIGNAL" && !k.starts_with("ONSIGNAL^"));
        Ok(CnvContent::Behavior(Behavior::from_initial_properties(
            parent,
            BehaviorProperties {
                code,
                condition,
                on_done,
                on_init,
                on_signal,
            },
        )))
    }
}

impl Initable for Behavior {
    fn initialize(&mut self, context: RunnerContext) -> RunnerResult<()> {
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                    arguments: Vec::new(),
                });
                if context.current_object.name == "__INIT__" {
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifierOwned::Method("RUN".into()),
                        arguments: Vec::new(),
                    });
                }
            });
        Ok(())
    }
}

impl BehaviorState {
    pub fn break_run(&self) -> RunnerResult<()> {
        // BREAK
        todo!()
    }

    pub fn run(&self, behavior: &Behavior, context: RunnerContext) -> RunnerResult<()> {
        // RUN
        if let Some(condition) = behavior.condition.as_ref() {
            let condition_object = context.runner.get_object(condition).unwrap();
            let condition_guard = condition_object.content.borrow();
            let condition: Option<&Condition> = (&*condition_guard).into();
            if let Some(condition) = condition {
                if !condition.check()? {
                    return Ok(());
                }
            } else {
                let condition: Option<&ComplexCondition> = (&*condition_guard).into(); // TODO: generalize
                let condition = condition.unwrap();
                if !condition.check()? {
                    return Ok(());
                }
            }
        }
        if let Some(v) = behavior.code.as_ref() {
            v.run(context)
        } else {
            Ok(())
        }
    }

    pub fn disable(&mut self) -> RunnerResult<()> {
        // DISABLE
        self.is_enabled = false;
        Ok(())
    }

    pub fn run_c(&self) -> RunnerResult<()> {
        // RUNC
        todo!()
    }

    pub fn run_looped(&self) -> RunnerResult<()> {
        // RUNLOOPED
        todo!()
    }
}
