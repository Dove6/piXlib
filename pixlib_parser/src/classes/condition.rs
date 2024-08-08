use std::any::Any;

use parsers::{discard_if_empty, parse_program, ConditionOperator};

use crate::runner::RunnerError;

use super::*;

#[derive(Debug, Clone)]
pub struct ConditionInit {
    // CONDITION
    pub operand1: Option<VariableName>,      // OPERAND1
    pub operand2: Option<VariableName>,      // OPERAND2
    pub operator: Option<ConditionOperator>, // OPERATOR

    pub on_runtime_failed: Option<Arc<IgnorableProgram>>, // ONRUNTIMEFAILED signal
    pub on_runtime_success: Option<Arc<IgnorableProgram>>, // ONRUNTIMESUCCESS signal
}

#[derive(Debug, Clone)]
pub struct Condition {
    // CONDITION
    pub parent: Arc<CnvObject>,
    pub initial_properties: ConditionInit,
}

impl Condition {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: ConditionInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn break_running(&self) {
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
        let left = runner
            .get_object(left)
            .and_then(|o| {
                o.call_method(
                    CallableIdentifier::Method("GET"),
                    &Vec::new(),
                    Some(context.clone()),
                )
                .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| CnvValue::String(left.clone()));
        let right = runner
            .get_object(right)
            .and_then(|o| {
                o.call_method(
                    CallableIdentifier::Method("GET"),
                    &Vec::new(),
                    Some(context.clone()),
                )
                .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| CnvValue::String(right.clone()));
        let Some(operator) = &self.initial_properties.operator else {
            return Err(RunnerError::MissingOperator {
                object_name: self.parent.name.clone(),
            });
        };
        let result = match operator {
            ConditionOperator::Equal => Ok(&left == &right),
            ConditionOperator::NotEqual => Ok(&left != &right),
            ConditionOperator::Less => Ok(left.to_double() < right.to_double()), // TODO: handle integers
            ConditionOperator::LessEqual => Ok(left.to_double() <= right.to_double()),
            ConditionOperator::Greater => Ok(left.to_double() > right.to_double()),
            ConditionOperator::GreaterEqual => Ok(left.to_double() >= right.to_double()),
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

    pub fn one_break(&self) {
        todo!()
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
        // eprintln!(
        //     "Calling method {:?} of condition {}",
        //     name, self.parent.name
        // );
        match name {
            CallableIdentifier::Method("CHECK") => self.check().map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Event("ONRUNTIMEFAILED") => {
                if let Some(v) = self.initial_properties.on_runtime_failed.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONRUNTIMESUCCESS") => {
                if let Some(v) = self.initial_properties.on_runtime_success.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "ONRUNTIMEFAILED" => self
                .initial_properties
                .on_runtime_failed
                .clone()
                .map(|v| v.into()),
            "ONRUNTIMESUCCESS" => self
                .initial_properties
                .on_runtime_success
                .clone()
                .map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let map = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ConditionOperator::parse);
        let operator = map.transpose()?;
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
        Ok(Condition::from_initial_properties(
            parent,
            ConditionInit {
                operand1,
                operand2,
                operator,
                on_runtime_failed,
                on_runtime_success,
            },
        ))
    }
}
