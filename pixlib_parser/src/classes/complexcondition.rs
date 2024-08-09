use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_program, ComplexConditionOperator};

use crate::ast::ParsedScript;

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
            state: RefCell::new(ComplexConditionState {
                ..Default::default()
            }),
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

    fn has_event(&self, name: &str) -> bool {
        matches!(name, "ONRUNTIMEFAILED" | "ONRUNTIMESUCCESS")
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
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
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

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
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
            .map(parse_program)
            .transpose()?;
        let on_runtime_success = properties
            .remove("ONRUNTIMESUCCESS")
            .and_then(discard_if_empty)
            .map(parse_program)
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
        let runner = Arc::clone(&complex_condition.parent.parent.runner);
        let context = RunnerContext {
            runner: Arc::clone(&runner),
            self_object: complex_condition.parent.name.clone(),
            current_object: complex_condition.parent.name.clone(),
        };
        let left_object = runner.get_object(&complex_condition.left).unwrap();
        let left_guard = left_object.content.borrow();
        let left: Option<&Condition> = (&*left_guard).into();
        let left = left.unwrap();
        let right_object = runner.get_object(&complex_condition.right).unwrap();
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
                complex_condition.call_method(
                    CallableIdentifier::Event("ONRUNTIMEFAILED"),
                    &Vec::new(),
                    context,
                )?;
            }
            Ok(true) => {
                complex_condition.call_method(
                    CallableIdentifier::Event("ONRUNTIMESUCCESS"),
                    &Vec::new(),
                    context,
                )?;
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
