use std::any::Any;

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
    pub parent: Arc<RwLock<CnvObject>>,
    pub initial_properties: ConditionInit,
}

impl Condition {
    pub fn from_initial_properties(
        parent: Arc<RwLock<CnvObject>>,
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

    pub fn check(&mut self, context: &mut RunnerContext) -> RunnerResult<bool> {
        let Some(left) = &self.initial_properties.operand1 else {
            return Err(RunnerError::MissingLeftOperand);
        };
        let Some(right) = &self.initial_properties.operand2 else {
            return Err(RunnerError::MissingRightOperand);
        };
        let left = context
            .runner
            .get_object(left)
            .map(|o| {
                o.write().unwrap().call_method(
                    CallableIdentifier::Method("GET"),
                    &Vec::new(),
                    context,
                )
            })
            .transpose()?
            .unwrap()
            .unwrap_or_else(|| CnvValue::String(left.clone()));
        let right = context
            .runner
            .get_object(right)
            .map(|o| {
                o.write().unwrap().call_method(
                    CallableIdentifier::Method("GET"),
                    &Vec::new(),
                    context,
                )
            })
            .transpose()?
            .unwrap()
            .unwrap_or_else(|| CnvValue::String(right.clone()));
        let Some(operator) = &self.initial_properties.operator else {
            return Err(RunnerError::MissingOperator);
        };
        let result = match operator {
            ConditionOperator::Equal => Ok(&left == &right),
            _ => todo!(),
        };
        match result {
            Ok(false) => {
                self.call_method(
                    CallableIdentifier::Event("ONRUNTIMEFAILED"),
                    &Vec::new(),
                    context,
                );
            }
            Ok(true) => {
                self.call_method(
                    CallableIdentifier::Event("ONRUNTIMESUCCESS"),
                    &Vec::new(),
                    context,
                );
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
        &mut self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("CHECK") => {
                self.check(context).map(|v| Some(CnvValue::Boolean(v)))
            }
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
            _ => todo!(),
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
        parent: Arc<RwLock<CnvObject>>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ConditionOperator::parse)
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
