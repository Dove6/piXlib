use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_program, ExpressionOperator};

use crate::{ast::ParsedScript, runner::CnvExpression};

use super::*;

#[derive(Debug, Clone)]
pub struct ExpressionProperties {
    // EXPRESSION
    pub operand1: Arc<ParsedScript>,  // OPERAND1
    pub operand2: Arc<ParsedScript>,  // OPERAND2
    pub operator: ExpressionOperator, // OPERATOR
}

#[derive(Debug, Clone, Default)]
pub struct ExpressionState {}

#[derive(Debug, Clone)]
pub struct ExpressionEventHandlers {}

#[derive(Debug, Clone)]
pub struct Expression {
    parent: Arc<CnvObject>,

    state: RefCell<ExpressionState>,
    event_handlers: ExpressionEventHandlers,

    operator: ExpressionOperator,
    left: Arc<ParsedScript>,
    right: Arc<ParsedScript>,
}

impl Expression {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: ExpressionProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(ExpressionState {
                ..Default::default()
            }),
            event_handlers: ExpressionEventHandlers {},
            operator: props.operator,
            left: props.operand1,
            right: props.operand2,
        }
    }

    ///

    pub fn calculate(&self) -> RunnerResult<CnvValue> {
        let runner = Arc::clone(&self.parent.parent.runner);
        let context = RunnerContext {
            runner: Arc::clone(&runner),
            self_object: self.parent.name.clone(),
            current_object: self.parent.name.clone(),
        };
        let left = self.left.calculate(context.clone())?.unwrap();
        let right = self.right.calculate(context.clone())?.unwrap();
        Ok(match self.operator {
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
            .map(ExpressionOperator::parse)
            .transpose()?
            .ok_or(TypeParsingError::MissingOperator)?;
        Ok(CnvContent::Expression(Self::from_initial_properties(
            parent,
            ExpressionProperties {
                operand1,
                operand2,
                operator,
            },
        )))
    }
}
