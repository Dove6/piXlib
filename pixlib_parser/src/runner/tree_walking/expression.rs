use crate::{
    parser::ast::{Expression, IgnorableExpression, Invocation, Operation},
    runner::{CallableIdentifier, RunnerError},
};

use super::super::{CnvStatement, CnvValue, RunnerContext};

pub trait CnvExpression {
    fn calculate(&self, context: RunnerContext) -> anyhow::Result<CnvValue>;
}

impl CnvExpression for IgnorableExpression {
    fn calculate(&self, context: RunnerContext) -> anyhow::Result<CnvValue> {
        // println!("IgnorableExpression::calculate: {:?}", self);
        if self.ignored {
            Ok(CnvValue::Null)
        } else {
            self.value.calculate(context)
        }
    }
}

fn substitute_behavior_arguments(identifier: &str, context: &RunnerContext) -> String {
    let mut needle = String::from("$1");
    let mut haystack = identifier.to_owned();
    let argument_count: u8 = context.arguments.len().min(9) as u8;
    for i in 0..argument_count {
        needle.drain(1..);
        needle.push((b'1' + i) as char);
        haystack = identifier.replace(&needle, &context.arguments[i as usize].to_str());
    }
    haystack
}

impl CnvExpression for Expression {
    fn calculate(&self, context: RunnerContext) -> anyhow::Result<CnvValue> {
        // println!("Expression::calculate: {:?} with context: {}", self, context);
        let result = match self {
            Expression::LiteralBool(b) => Ok(CnvValue::Bool(*b)),
            Expression::LiteralNull => Ok(CnvValue::Null),
            Expression::Identifier(name) => Ok(CnvValue::String(substitute_behavior_arguments(
                name, &context,
            ))),
            Expression::Invocation(invocation) => invocation.calculate(context.clone()),
            Expression::SelfReference => Ok(CnvValue::String(context.self_object.name.clone())),
            Expression::Parameter(name) => Ok(context
                .arguments
                .get(name.parse::<usize>().unwrap() - 1)
                .expect("Expected argument")
                .clone()),
            Expression::NameResolution(expression) => {
                let name = &expression.calculate(context.clone())?;
                let name = name.to_str();
                Ok(CnvValue::String(name))
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
                Ok(result)
            }
            Expression::Block(block) => {
                // TODO: create an anonymous function object
                // TODO: handle arguments and return
                for statement in block {
                    statement.run(context.clone())?;
                }
                Ok(CnvValue::Null)
            }
        };
        // println!("    result: {:?}", result);
        result
    }
}

impl CnvExpression for Invocation {
    fn calculate(&self, context: RunnerContext) -> anyhow::Result<CnvValue> {
        // println!("Invocation::calculate: {:?} with context {}", self, context);
        if self.parent.is_none() {
            Ok(CnvValue::Null) // TODO: match &self.name
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
                .collect::<anyhow::Result<Vec<_>>>()?;
            let arguments: Vec<_> = arguments.into_iter().collect();
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
