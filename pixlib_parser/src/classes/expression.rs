use std::any::Any;

use parsers::{discard_if_empty, ExpressionOperator};

use super::*;

#[derive(Debug, Clone)]
pub struct ExpressionInit {
    // EXPRESSION
    pub operand1: Option<VariableName>,       // OPERAND1
    pub operand2: Option<VariableName>,       // OPERAND2
    pub operator: Option<ExpressionOperator>, // OPERATOR
}

#[derive(Debug, Clone)]
pub struct Expression {
    parent: Arc<CnvObject>,
    initial_properties: ExpressionInit,
}

impl Expression {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: ExpressionInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }
}

impl CnvType for Expression {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "EXPRESSION"
    }

    fn has_event(&self, _name: &str) -> bool {
        false
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
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ExpressionOperator::parse)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            ExpressionInit {
                operand1,
                operand2,
                operator,
            },
        ))
    }
}
