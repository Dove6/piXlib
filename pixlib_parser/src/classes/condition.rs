use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_program, ConditionOperator};

use crate::{
    ast::ParsedScript,
    common::DroppableRefMut,
    runner::{CnvExpression, InternalEvent},
};

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
            state: RefCell::new(ConditionState {
                ..Default::default()
            }),
            event_handlers: ConditionEventHandlers {
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
        _arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // eprintln!(
        //     "Calling method {:?} of condition {}",
        //     name, self.parent.name
        // );
        match name {
            CallableIdentifier::Method("BREAK") => self.state.borrow().break_run().map(|_| None),
            CallableIdentifier::Method("CHECK") => self
                .state
                .borrow()
                .check(self)
                .map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Method("ONE_BREAK") => {
                self.state.borrow().one_break().map(|_| None)
            }
            CallableIdentifier::Event("ONRUNTIMEFAILED") => {
                if let Some(v) = self.event_handlers.on_runtime_failed.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONRUNTIMESUCCESS") => {
                if let Some(v) = self.event_handlers.on_runtime_success.as_ref() {
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
            .map(parse_program)
            .transpose()?;
        let on_runtime_success = properties
            .remove("ONRUNTIMESUCCESS")
            .and_then(discard_if_empty)
            .map(parse_program)
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
    pub fn break_run(&self) -> RunnerResult<()> {
        todo!()
    }

    pub fn check(&self, condition: &Condition) -> RunnerResult<bool> {
        let context =
            RunnerContext::new_minimal(&condition.parent.parent.runner, &condition.parent);
        let left = condition.left.calculate(context.clone())?.unwrap();
        let right = condition.right.calculate(context.clone())?.unwrap();
        let result = match condition.operator {
            ConditionOperator::Equal => Ok(&left == &right),
            ConditionOperator::NotEqual => Ok(&left != &right),
            ConditionOperator::Less => Ok(left.to_double() < right.to_double()), // TODO: handle integers
            ConditionOperator::LessEqual => Ok(left.to_double() <= right.to_double()),
            ConditionOperator::Greater => Ok(left.to_double() > right.to_double()),
            ConditionOperator::GreaterEqual => Ok(left.to_double() >= right.to_double()),
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
        todo!()
    }
}
