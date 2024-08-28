use crate::parser::ast::{ParsedScript, Statement};

use super::super::{CnvExpression, RunnerContext};

pub trait CnvStatement {
    fn run(&self, context: RunnerContext) -> anyhow::Result<()>;
}

impl CnvStatement for Statement {
    fn run(&self, context: RunnerContext) -> anyhow::Result<()> {
        // log::trace!("Statement::run: {:?}", self);
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
        // log::trace!("ParsedScript::run: {:?}", self);
        self.calculate(context)?;
        Ok(())
    }
}
