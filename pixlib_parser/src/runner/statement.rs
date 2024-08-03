use crate::{
    ast::{IgnorableProgram, IgnorableStatement, Program, Statement},
    classes::CallableIdentifier,
};

use super::{CnvExpression, RunnerContext};

pub trait CnvStatement {
    fn run(&self, context: &mut RunnerContext);
}

impl CnvStatement for IgnorableProgram {
    fn run(&self, context: &mut RunnerContext) {
        // println!("IgnorableProgram::run: {:?}", self);
        if self.ignored {
            return;
        }
        self.value.run(context);
    }
}

impl CnvStatement for Program {
    fn run(&self, context: &mut RunnerContext) {
        // println!("Program::run: {:?}", self);
        match self {
            Program::Identifier(identifier) => {
                let obj = context
                    .runner
                    .get_object(identifier)
                    .unwrap_or_else(|| panic!("Expected existing object named {}", &identifier));
                obj.call_method(CallableIdentifier::Method("RUN"), &Vec::new(), context);
            }
            Program::Block(ignorable_statements) => {
                for ignorable_statement in ignorable_statements {
                    ignorable_statement.run(context);
                }
            }
        }
    }
}

impl CnvStatement for IgnorableStatement {
    fn run(&self, context: &mut RunnerContext) {
        // println!("IgnorableStatement::run: {:?}", self);
        if self.ignored {
            return;
        }
        self.value.run(context);
    }
}

impl CnvStatement for Statement {
    fn run(&self, context: &mut RunnerContext) {
        // println!("Statement::run: {:?}", self);
        match self {
            Statement::Invocation(invocation) => {
                invocation.calculate(context);
            }
            Statement::ExpressionStatement(expression) => {
                expression.calculate(context);
            }
        }
    }
}
