use std::any::Any;

use parsers::{discard_if_empty, parse_program, ComplexConditionOperator};

use crate::{ast::ParsedScript, runner::RunnerError};

use super::*;

#[derive(Debug, Clone)]
pub struct ComplexConditionInit {
    // COMPLEXCONDITION
    pub operand1: Option<ConditionName>,            // OPERAND1
    pub operand2: Option<ConditionName>,            // OPERAND2
    pub operator: Option<ComplexConditionOperator>, // OPERATOR

    pub on_runtime_failed: Option<Arc<ParsedScript>>, // ONRUNTIMEFAILED signal
    pub on_runtime_success: Option<Arc<ParsedScript>>, // ONRUNTIMESUCCESS signal
}

#[derive(Debug, Clone)]
pub struct ComplexCondition {
    pub parent: Arc<CnvObject>,
    pub initial_properties: ComplexConditionInit,
}

impl ComplexCondition {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: ComplexConditionInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn break_running() {
        // BREAK
        todo!()
    }

    pub fn check(&self) -> RunnerResult<bool> {
        let Some(left) = &self.initial_properties.operand1 else {
            return Err(RunnerError::MissingLeftOperand {
                object_name: self.parent.name.clone(),
            });
        };
        let Some(right) = &self.initial_properties.operand2 else {
            return Err(RunnerError::MissingRightOperand {
                object_name: self.parent.name.clone(),
            });
        };
        let runner = Arc::clone(&self.parent.parent.runner);
        let context = RunnerContext {
            runner: Arc::clone(&runner),
            self_object: self.parent.name.clone(),
            current_object: self.parent.name.clone(),
        };
        let left_object = runner.get_object(left).unwrap();
        let left_guard = left_object.content.borrow();
        let left: Option<&Condition> = (&*left_guard).into();
        let left = left.unwrap();
        let right_object = runner.get_object(right).unwrap();
        let right_guard = right_object.content.borrow();
        let right: Option<&Condition> = (&*right_guard).into();
        let right = right.unwrap();
        let Some(operator) = &self.initial_properties.operator else {
            return Err(RunnerError::MissingOperator {
                object_name: self.parent.name.clone(),
            });
        };
        let result = match operator {
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
                self.call_method(
                    CallableIdentifier::Event("ONRUNTIMEFAILED"),
                    &Vec::new(),
                    context,
                )?;
            }
            Ok(true) => {
                self.call_method(
                    CallableIdentifier::Event("ONRUNTIMESUCCESS"),
                    &Vec::new(),
                    context,
                )?;
            }
            _ => {}
        }
        result
    }

    pub fn one_break() {
        // ONE_BREAK
        todo!()
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
            CallableIdentifier::Method("CHECK") => self.check().map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Event("ONRUNTIMEFAILED") => {
                if let Some(v) = self.initial_properties.on_runtime_failed.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONRUNTIMESUCCESS") => {
                if let Some(v) = self.initial_properties.on_runtime_success.as_ref() {
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
        let operand1 = properties.remove("CONDITION1").and_then(discard_if_empty);
        let operand2 = properties.remove("CONDITION2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ComplexConditionOperator::parse)
            .transpose()?;
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
            ComplexConditionInit {
                operand1,
                operand2,
                operator,
                on_runtime_failed,
                on_runtime_success,
            },
        )))
    }
}
