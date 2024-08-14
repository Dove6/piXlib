use std::{any::Any, cell::RefCell};

use content::EventHandler;
use parsers::{discard_if_empty, parse_event_handler, ComplexConditionOperator};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

use super::*;

#[derive(Debug, Clone)]
pub struct ComplexConditionProperties {
    // COMPLEXCONDITION
    pub operand1: ConditionName,            // OPERAND1
    pub operand2: ConditionName,            // OPERAND2
    pub operator: ComplexConditionOperator, // OPERATOR

    pub on_runtime_failed: Option<Arc<ParsedScript>>, // ONRUNTIMEFAILED signal
    pub on_runtime_success: Option<Arc<ParsedScript>>, // ONRUNTIMESUCCESS signal
}

#[derive(Debug, Clone, Default)]
pub struct ComplexConditionState {}

#[derive(Debug, Clone)]
pub struct ComplexConditionEventHandlers {
    pub on_runtime_failed: Option<Arc<ParsedScript>>, // ONRUNTIMEFAILED signal
    pub on_runtime_success: Option<Arc<ParsedScript>>, // ONRUNTIMESUCCESS signal
}

impl EventHandler for ComplexConditionEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONRUNTIMEFAILED" => self.on_runtime_failed.as_ref(),
            "ONRUNTIMESUCCESS" => self.on_runtime_success.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComplexCondition {
    pub parent: Arc<CnvObject>,

    pub state: RefCell<ComplexConditionState>,
    pub event_handlers: ComplexConditionEventHandlers,

    pub operator: ComplexConditionOperator,
    pub left: ConditionName,
    pub right: ConditionName,
}

impl ComplexCondition {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        props: ComplexConditionProperties,
    ) -> Self {
        Self {
            parent,
            state: RefCell::new(ComplexConditionState {}),
            event_handlers: ComplexConditionEventHandlers {
                on_runtime_failed: props.on_runtime_failed,
                on_runtime_success: props.on_runtime_success,
            },
            operator: props.operator,
            left: props.operand1,
            right: props.operand2,
        }
    }

    pub fn check(&self) -> RunnerResult<bool> {
        self.state.borrow().check(self)
    }
}

impl CnvType for ComplexCondition {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "COMPLEXCONDITION"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("BREAK") => self.state.borrow().break_run().map(|_| None),
            CallableIdentifier::Method("CHECK") => self
                .state
                .borrow()
                .check(self)
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("ONE_BREAK") => {
                self.state.borrow().one_break().map(|_| None)
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new_content(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        // eprintln!("Creating {} from properties: {:#?}", parent.name, properties);
        let operand1 = properties
            .remove("CONDITION1")
            .and_then(discard_if_empty)
            .ok_or(TypeParsingError::MissingLeftOperand)?;
        let operand2 = properties
            .remove("CONDITION2")
            .and_then(discard_if_empty)
            .ok_or(TypeParsingError::MissingRightOperand)?;
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ComplexConditionOperator::parse)
            .transpose()?
            .ok_or(TypeParsingError::MissingOperator)?;
        let on_runtime_failed = properties
            .remove("ONRUNTIMEFAILED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_runtime_success = properties
            .remove("ONRUNTIMESUCCESS")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::ComplexCondition(Self::from_initial_properties(
            parent,
            ComplexConditionProperties {
                operand1,
                operand2,
                operator,
                on_runtime_failed,
                on_runtime_success,
            },
        )))
    }
}

impl ComplexConditionState {
    pub fn break_run(&self) -> RunnerResult<()> {
        // BREAK
        todo!()
    }

    pub fn check(&self, complex_condition: &ComplexCondition) -> RunnerResult<bool> {
        let context = RunnerContext::new_minimal(
            &complex_condition.parent.parent.runner,
            &complex_condition.parent,
        );
        let left_object = context.runner.get_object(&complex_condition.left).unwrap();
        let left_guard = left_object.content.borrow();
        let left: Option<&Condition> = (&*left_guard).into();
        let left = left.unwrap();
        let right_object = context.runner.get_object(&complex_condition.right).unwrap();
        let right_guard = right_object.content.borrow();
        let right: Option<&Condition> = (&*right_guard).into();
        let right = right.unwrap();
        let result = match complex_condition.operator {
            ComplexConditionOperator::And => {
                if !left.check()? {
                    Ok(false)
                } else {
                    Ok(right.check()?)
                }
            }
            ComplexConditionOperator::Or => {
                if left.check()? {
                    Ok(true)
                } else {
                    Ok(right.check()?)
                }
            }
        };
        match result {
            Ok(false) => {
                context
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(move |events| {
                        events.push_back(InternalEvent {
                            object: context.current_object.clone(),
                            callable: CallableIdentifier::Event("ONRUNTIMEFAILED").to_owned(),
                            arguments: Vec::new(),
                        });
                    });
            }
            Ok(true) => {
                context
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(move |events| {
                        events.push_back(InternalEvent {
                            object: context.current_object.clone(),
                            callable: CallableIdentifier::Event("ONRUNTIMESUCCESS").to_owned(),
                            arguments: Vec::new(),
                        });
                    });
            }
            _ => {}
        }
        result
    }

    pub fn one_break(&self) -> RunnerResult<()> {
        // ONE_BREAK
        todo!()
    }
}
