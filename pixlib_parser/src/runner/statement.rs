use crate::ast::{ParsedScript, Statement};

use super::{CnvExpression, RunnerContext, RunnerResult};

pub trait CnvStatement {
    fn run(&self, context: RunnerContext) -> RunnerResult<()>;
}

impl CnvStatement for Statement {
    fn run(&self, context: RunnerContext) -> RunnerResult<()> {
        println!("Statement::run: {:?}", self);
        match self {
            Statement::ExpressionStatement(expression) => {
                expression.calculate(context)?;
            }
        }
        Ok(())
    }
}

impl CnvStatement for ParsedScript {
    fn run(&self, context: RunnerContext) -> RunnerResult<()> {
        println!("ParsedScript::run: {:?}", self);
        self.calculate(context)?;
        Ok(())
    }
}
