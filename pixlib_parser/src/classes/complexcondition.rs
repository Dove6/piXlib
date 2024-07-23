use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct ComplexConditionInit {
    // COMPLEXCONDITION
    pub operand1: Option<ConditionName>,            // OPERAND1
    pub operand2: Option<ConditionName>,            // OPERAND2
    pub operator: Option<ComplexConditionOperator>, // OPERATOR

    pub on_runtime_failed: Option<Arc<IgnorableProgram>>, // ONRUNTIMEFAILED signal
    pub on_runtime_success: Option<Arc<IgnorableProgram>>, // ONRUNTIMESUCCESS signal
}

#[derive(Debug, Clone)]
pub struct ComplexCondition {
    pub initial_properties: ComplexConditionInit,
}

impl ComplexCondition {
    pub fn from_initial_properties(initial_properties: ComplexConditionInit) -> Self {
        Self { initial_properties }
    }

    pub fn break_running() {
        // BREAK
        todo!()
    }

    pub fn check() {
        // CHECK
        todo!()
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

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(path: Arc<Path>, mut properties: HashMap<String, String>, filesystem: &dyn FileSystem) -> Result<Self, TypeParsingError> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
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
        Ok(Self::from_initial_properties(ComplexConditionInit {
            operand1,
            operand2,
            operator,
            on_runtime_failed,
            on_runtime_success,
        }))
    }
}
