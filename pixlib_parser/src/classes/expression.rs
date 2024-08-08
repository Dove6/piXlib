use std::any::Any;

use parsers::{discard_if_empty, parse_program, ExpressionOperator};

use crate::{
    ast::ParsedScript,
    runner::{CnvExpression, RunnerError},
};

use super::*;

#[derive(Debug, Clone)]
pub struct ExpressionInit {
    // EXPRESSION
    pub operand1: Option<Arc<ParsedScript>>,  // OPERAND1
    pub operand2: Option<Arc<ParsedScript>>,  // OPERAND2
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

    ///

    pub fn calculate(&self) -> RunnerResult<CnvValue> {
        let Some(operator) = self.initial_properties.operator.as_ref() else {
            return Err(RunnerError::MissingOperator {
                object_name: self.parent.name.clone(),
            });
        };
        let Some(left) = self.initial_properties.operand1.as_ref() else {
            return Err(RunnerError::MissingLeftOperand {
                object_name: self.parent.name.clone(),
            });
        };
        let Some(right) = self.initial_properties.operand2.as_ref() else {
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
        let left = left.calculate(context.clone())?.unwrap();
        let right = right.calculate(context.clone())?.unwrap();
        Ok(match operator {
            ExpressionOperator::Add => &left + &right,
            ExpressionOperator::Sub => &left - &right,
            ExpressionOperator::Mul => &left * &right,
            ExpressionOperator::Div => &left / &right,
            ExpressionOperator::Mod => &left % &right,
        })
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
    ) -> Result<CnvContent, TypeParsingError> {
        let operand1 = properties
            .remove("OPERAND1")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let operand2 = properties
            .remove("OPERAND2")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ExpressionOperator::parse)
            .transpose()?;
        Ok(CnvContent::Expression(Self::from_initial_properties(
            parent,
            ExpressionInit {
                operand1,
                operand2,
                operator,
            },
        )))
    }
}
