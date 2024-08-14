use crate::{
    ast::{Expression, IgnorableExpression, Invocation, Operation},
    classes::CallableIdentifier,
    runner::RunnerError,
};

use super::{value::CnvValue, CnvStatement, RunnerContext, RunnerResult};

pub trait CnvExpression {
    fn calculate(&self, context: RunnerContext) -> RunnerResult<Option<CnvValue>>;
}

impl CnvExpression for IgnorableExpression {
    fn calculate(&self, context: RunnerContext) -> RunnerResult<Option<CnvValue>> {
        // println!("IgnorableExpression::calculate: {:?}", self);
        if self.ignored {
            return Ok(None);
        }
        self.value.calculate(context)
    }
}

impl CnvExpression for Expression {
    fn calculate(&self, context: RunnerContext) -> RunnerResult<Option<CnvValue>> {
        // println!("Expression::calculate: {:?} with context: {}", self, context);
        let result = match self {
            Expression::LiteralBool(b) => Ok(Some(CnvValue::Bool(*b))),
            Expression::Identifier(name) => Ok(Some(CnvValue::String(name.to_owned()))),
            Expression::Invocation(invocation) => invocation.calculate(context.clone()),
            Expression::SelfReference => {
                Ok(Some(CnvValue::String(context.self_object.name.clone())))
            }
            Expression::Parameter(name) => Ok(context
                .arguments
                .get(name.parse::<usize>().unwrap() - 1)
                .cloned()),
            Expression::NameResolution(expression) => {
                let name = &expression.calculate(context.clone())?.unwrap();
                let name = name.to_str();
                Ok(Some(CnvValue::String(name)))
            }
            Expression::FieldAccess(_expression, _field) => todo!(),
            Expression::Operation(expression, operations) => {
                let mut result = expression
                    .calculate(context.clone())?
                    .expect("Expected non-void argument in operation");
                for (operation, argument) in operations {
                    let argument = argument
                        .calculate(context.clone())?
                        .expect("Expected non-void argument in operation");
                    result = match operation {
                        Operation::Addition => &result + &argument,
                        Operation::Multiplication => &result * &argument,
                        Operation::Subtraction => &result - &argument,
                        Operation::Division => &result / &argument,
                        Operation::Remainder => &result % &argument,
                    }
                }
                Ok(Some(result))
            }
            Expression::Block(block) => {
                // TODO: create an anonymous function object
                // TODO: handle arguments and return
                for statement in block {
                    statement.run(context.clone())?;
                }
                Ok(None)
            }
        };
        // println!("    result: {:?}", result);
        result
    }
}

impl CnvExpression for Invocation {
    fn calculate(&self, context: RunnerContext) -> RunnerResult<Option<CnvValue>> {
        // println!("Invocation::calculate: {:?} with context {}", self, context);
        if self.parent.is_none() {
            Ok(None) // TODO: match &self.name
        } else {
            let parent = self
                .parent
                .as_ref()
                .unwrap()
                .calculate(context.clone())?
                .expect("Invalid invocation parent");
            let arguments = self
                .arguments
                .iter()
                .map(|e| e.calculate(context.clone()))
                .collect::<RunnerResult<Vec<_>>>()?;
            let arguments: Vec<_> = arguments.into_iter().map(|e| e.unwrap()).collect();
            // println!("Calling method: {:?} of: {:?}", self.name, self.parent);
            let name = parent.to_str();
            context
                .runner
                .get_object(&name)
                .ok_or(RunnerError::ObjectNotFound { name })?
                .call_method(
                    CallableIdentifier::Method(&self.name),
                    &arguments,
                    Some(context.with_arguments(arguments.clone())),
                )
        }
    }
}
