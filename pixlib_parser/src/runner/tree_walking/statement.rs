use crate::parser::ast::{ParsedScript, Statement};

use super::super::{CnvExpression, RunnerContext};

pub trait CnvStatement {
    fn run(&self, context: RunnerContext) -> anyhow::Result<()>;
}

impl CnvStatement for Statement {
    fn run(&self, context: RunnerContext) -> anyhow::Result<()> {
        // println!("Statement::run: {:?}", self);
        match self {
            Statement::ExpressionStatement(expression) => {
                expression.calculate(context)?;
            }
        }
        Ok(())
    }
}

impl CnvStatement for ParsedScript {
    fn run(&self, context: RunnerContext) -> anyhow::Result<()> {
        // println!("ParsedScript::run: {:?}", self);
        self.calculate(context)?;
        Ok(())
    }
}
