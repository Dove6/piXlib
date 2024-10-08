use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler, parse_program};

use crate::{
    common::DroppableRefMut,
    parser::ast::ParsedScript,
    runner::{CnvExpression, InternalEvent},
};

use super::super::common::*;
use super::super::*;
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

impl EventHandler for BehaviorEventHandlers {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => argument
                .and_then(|a| self.on_signal.get(a))
                .or(self.on_signal.get("")),
            _ => None,
        }
    }
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
            state: RefCell::new(BehaviorState { is_enabled: true }),
            event_handlers: BehaviorEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
            },
            code: props.code,
            condition: props.condition,
        }
    }

    pub fn run(
        &self,
        context: RunnerContext,
        arguments: Vec<CnvValue>,
    ) -> anyhow::Result<CnvValue> {
        if let Some(code) = self.code.as_ref() {
            self.state.borrow().run(context, code.clone(), arguments)
        } else {
            Ok(CnvValue::Null)
        }
    }

    pub fn run_c(
        &self,
        context: RunnerContext,
        arguments: Vec<CnvValue>,
    ) -> anyhow::Result<CnvValue> {
        if let Some(code) = self.code.as_ref() {
            self.state
                .borrow()
                .run_c(context, code.clone(), self.condition.as_deref(), arguments)
        } else {
            Ok(CnvValue::Null)
        }
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
    ) -> anyhow::Result<CnvValue> {
        // log::trace!("Calling method: {:?} of object: {:?}", name, self);
        let context = context.with_current_object(self.parent.clone());
        match name {
            CallableIdentifier::Method("BREAK") => {
                self.state.borrow().break_run().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DISABLE") => {
                self.state.borrow_mut().disable().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RUN") => {
                if let Some(code) = self.code.as_ref() {
                    self.state
                        .borrow()
                        .run(context, code.clone(), arguments.to_owned())
                        .map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
            }
            CallableIdentifier::Method("RUNC") => {
                if let Some(code) = self.code.as_ref() {
                    self.state
                        .borrow()
                        .run_c(
                            context,
                            code.clone(),
                            self.condition.as_deref(),
                            arguments.to_owned(),
                        )
                        .map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
            }
            CallableIdentifier::Method("RUNLOOPED") => {
                if let Some(code) = self.code.as_ref() {
                    self.state
                        .borrow()
                        .run_looped(
                            context,
                            code.clone(),
                            arguments[0].to_int(),
                            arguments[1].to_int(),
                            arguments.get(2).map(CnvValue::to_int).unwrap_or(1),
                        )
                        .map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context).map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
            }
            ident => Err(RunnerError::InvalidCallable {
                object_name: self.parent.name.clone(),
                callable: ident.to_owned(),
            }
            .into()),
        }
    }

    fn new_content(
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
            .map(parse_event_handler)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_signal = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSIGNAL" {
                on_signal.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONSIGNAL^") {
                on_signal.insert(String::from(argument), parse_event_handler(v.to_owned())?);
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
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(Vec::new()),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                });
                if context.current_object.name == "__INIT__" {
                    events.push_back(InternalEvent {
                        context: context.clone().with_arguments(Vec::new()),
                        callable: CallableIdentifierOwned::Method("RUN".into()),
                    });
                }
            });
        Ok(())
    }
}

impl BehaviorState {
    pub fn break_run(&self) -> anyhow::Result<()> {
        // BREAK
        todo!()
    }

    pub fn run(
        &self,
        context: RunnerContext,
        code: Arc<ParsedScript>,
        arguments: Vec<CnvValue>,
    ) -> anyhow::Result<CnvValue> {
        // RUN
        // log::trace!(
        //     "Running behavior {} with arguments [{}]",
        //     context.current_object.name,
        //     arguments.iter().join(", ")
        // );
        let context = context.with_arguments(arguments);
        code.calculate(context)
    }

    pub fn disable(&mut self) -> anyhow::Result<()> {
        // DISABLE
        self.is_enabled = false;
        Ok(())
    }

    pub fn run_c(
        &self,
        context: RunnerContext,
        code: Arc<ParsedScript>,
        condition_name: Option<&str>,
        arguments: Vec<CnvValue>,
    ) -> anyhow::Result<CnvValue> {
        // RUNC
        if let Some(condition) = condition_name {
            let condition_object = context.runner.get_object(condition).unwrap();
            let condition: &dyn GeneralCondition = match &condition_object.content {
                CnvContent::Condition(c) => c,
                CnvContent::ComplexCondition(c) => c,
                _ => return Err(RunnerError::ExpectedConditionObject.into()),
            };
            if !condition.check(Some(context.clone()))? {
                return Ok(CnvValue::Null);
            }
        }
        self.run(context, code, arguments)
    }

    pub fn run_looped(
        &self,
        context: RunnerContext,
        code: Arc<ParsedScript>,
        start: i32,
        width: i32,
        step: i32,
    ) -> anyhow::Result<()> {
        // RUNLOOPED
        if step < 0 {
            todo!();
        }
        for i in (start..(start + width)).step_by(step as usize) {
            self.run(context.clone(), code.clone(), vec![CnvValue::Integer(i)])?;
        }
        Ok(())
    }
}
