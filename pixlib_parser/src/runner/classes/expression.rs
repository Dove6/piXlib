use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::parsers::{discard_if_empty, parse_program, ExpressionOperator};

use crate::{
    parser::ast::{self, ParsedScript},
    runner::CnvExpression,
};

use super::super::common::*;
use super::super::*;
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

impl EventHandler for ExpressionEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

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
            state: RefCell::new(ExpressionState {}),
            event_handlers: ExpressionEventHandlers {},
            operator: props.operator,
            left: props.operand1,
            right: props.operand2,
        }
    }

    // custom

    pub fn calculate(&self) -> anyhow::Result<CnvValue> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        let left = self
            .left
            .calculate(context.clone())?
            .map(|v| {
                if let ast::Expression::Identifier(_) = &self.left.value {
                    v.resolve(context.clone())
                } else {
                    v
                }
            })
            .unwrap();
        let right = self
            .right
            .calculate(context.clone())?
            .map(|v| {
                if let ast::Expression::Identifier(_) = &self.right.value {
                    v.resolve(context.clone())
                } else {
                    v
                }
            })
            .unwrap();
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<Option<CnvValue>> {
        match name {
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context)?;
                }
                Ok(None)
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
