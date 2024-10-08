use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::parsers::{
    discard_if_empty, parse_event_handler, parse_program, ConditionOperator,
};

use crate::{
    common::DroppableRefMut,
    parser::ast::{self, ParsedScript},
    runner::{CnvExpression, InternalEvent},
};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ConditionProperties {
    // CONDITION
    pub operand1: Arc<ParsedScript>, // OPERAND1
    pub operand2: Arc<ParsedScript>, // OPERAND2
    pub operator: ConditionOperator, // OPERATOR

    pub on_runtime_failed: Option<Arc<ParsedScript>>, // ONRUNTIMEFAILED signal
    pub on_runtime_success: Option<Arc<ParsedScript>>, // ONRUNTIMESUCCESS signal
}
#[derive(Debug, Clone, Default)]
pub struct ConditionState {}

#[derive(Debug, Clone)]
pub struct ConditionEventHandlers {
    pub on_runtime_failed: Option<Arc<ParsedScript>>, // ONRUNTIMEFAILED signal
    pub on_runtime_success: Option<Arc<ParsedScript>>, // ONRUNTIMESUCCESS signal
}

impl EventHandler for ConditionEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONRUNTIMEFAILED" => self.on_runtime_failed.as_ref(),
            "ONRUNTIMESUCCESS" => self.on_runtime_success.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    // CONDITION
    pub parent: Arc<CnvObject>,

    pub state: RefCell<ConditionState>,
    pub event_handlers: ConditionEventHandlers,

    pub operator: ConditionOperator,
    pub left: Arc<ParsedScript>,
    pub right: Arc<ParsedScript>,
}

impl Condition {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: ConditionProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(ConditionState {}),
            event_handlers: ConditionEventHandlers {
                on_runtime_failed: props.on_runtime_failed,
                on_runtime_success: props.on_runtime_success,
            },
            operator: props.operator,
            left: props.operand1,
            right: props.operand2,
        }
    }
}

impl GeneralCondition for Condition {
    fn check(&self, context: Option<RunnerContext>) -> anyhow::Result<bool> {
        let context = context
            .map(|c| c.with_current_object(self.parent.clone()))
            .unwrap_or(RunnerContext::new_minimal(
                &self.parent.parent.runner,
                &self.parent,
            ));
        self.state.borrow().check(context)
    }
}

impl CnvType for Condition {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "CONDITION"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        // log::trace!(
        //     "Calling method {:?} of condition {}",
        //     name, self.parent.name
        // );
        match name {
            CallableIdentifier::Method("BREAK") => self
                .state
                .borrow()
                .break_run(context, arguments[0].to_bool())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("CHECK") => {
                self.state.borrow().check(context).map(CnvValue::Bool)
            }
            CallableIdentifier::Method("ONE_BREAK") => self
                .state
                .borrow()
                .one_break(context, arguments[0].to_bool())
                .map(|_| CnvValue::Null),
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
        let operand1 = properties
            .remove("OPERAND1")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?
            .ok_or(TypeParsingError::MissingLeftOperand)?;
        let operand2 = properties
            .remove("OPERAND2")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?
            .ok_or(TypeParsingError::MissingRightOperand)?;
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ConditionOperator::parse)
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
        Ok(CnvContent::Condition(Condition::from_initial_properties(
            parent,
            ConditionProperties {
                operand1,
                operand2,
                operator,
                on_runtime_failed,
                on_runtime_success,
            },
        )))
    }
}

impl ConditionState {
    pub fn break_run(&self, context: RunnerContext, _: bool) -> anyhow::Result<()> {
        if self.check(context)? {
            Err(RunnerError::ExecutionInterrupted { one: false }.into())
        } else {
            Ok(())
        }
    }

    pub fn check(&self, context: RunnerContext) -> anyhow::Result<bool> {
        let CnvContent::Condition(ref condition) = &context.current_object.content else {
            panic!();
        };
        let left = condition.left.calculate(context.clone())?;
        let left = if let ast::Expression::Identifier(_) = &condition.left.value {
            left.resolve(context.clone())
        } else {
            left
        };
        let right = condition.right.calculate(context.clone())?;
        let right = if let ast::Expression::Identifier(_) = &condition.right.value {
            right.resolve(context.clone())
        } else {
            right
        };
        let result = match condition.operator {
            ConditionOperator::Equal => Ok(left == right),
            ConditionOperator::NotEqual => Ok(left != right),
            ConditionOperator::Less => Ok(left.to_dbl() < right.to_dbl()), // TODO: handle integers
            ConditionOperator::LessEqual => Ok(left.to_dbl() <= right.to_dbl()),
            ConditionOperator::Greater => Ok(left.to_dbl() > right.to_dbl()),
            ConditionOperator::GreaterEqual => Ok(left.to_dbl() >= right.to_dbl()),
        };
        let evt_context = context.clone().with_arguments(Vec::new());
        match result {
            Ok(false) => {
                context
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(move |events| {
                        events.push_back(InternalEvent {
                            context: evt_context,
                            callable: CallableIdentifier::Event("ONRUNTIMEFAILED").to_owned(),
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
                            context: evt_context,
                            callable: CallableIdentifier::Event("ONRUNTIMESUCCESS").to_owned(),
                        });
                    });
            }
            _ => {}
        }
        result
    }

    pub fn one_break(&self, context: RunnerContext, _: bool) -> anyhow::Result<()> {
        if self.check(context)? {
            Err(RunnerError::ExecutionInterrupted { one: true }.into())
        } else {
            Ok(())
        }
    }
}
