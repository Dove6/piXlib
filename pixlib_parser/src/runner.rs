use std::{collections::HashMap, ops::{Add, Mul, Sub, Div, Rem}};

use crate::ast::{IgnorableProgram, Program, IgnorableStatement, Expression, Invocation, Operation, Statement};

#[allow(dead_code)]
struct CnvApplication {
    application_name: String,
}

#[allow(dead_code)]
impl CnvApplication {
//     @BOOL
// @BREAK
// @CONTINUE
// @CONV
// @CREATE
// @DOUBLE
// @FOR
// @GETAPPLICATIONNAME
// @GETCURRENTSCENE
// @IF
// @INT
// @LOOP
// @MSGBOX
// @ONEBREAK
// @RETURN
// @RUNONTIMER
// @STRING
// @VALUE
// @WHILE

    fn get_application_name(&self) -> &str {
        &self.application_name
    }

}

pub trait CnvObject {
    fn call_method(&mut self, name: &String) -> Option<CnvValue>;
    fn get_value(&self) -> Option<CnvValue>;
}

pub enum CnvValue {
    Integer(i32),
    Double(f64),
    Boolean(bool),
    String(String),
}

impl CnvValue {
    pub fn to_integer(&self) -> i32 {
        match self {
            CnvValue::Integer(i) => *i,
            CnvValue::Double(d) => *d as i32,
            CnvValue::Boolean(b) => if *b { 1 } else { 0 },
            CnvValue::String(_) => 0,
        }
    }

    pub fn to_double(&self) -> f64 {
        match self {
            CnvValue::Integer(i) => (*i).into(),
            CnvValue::Double(d) => *d,
            CnvValue::Boolean(b) => if *b { 1.0 } else { 0.0 },
            CnvValue::String(_) => 0.0,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            CnvValue::Integer(i) => *i != 0, // TODO: check
            CnvValue::Double(d) => *d != 0.0,  // TODO: check
            CnvValue::Boolean(b) => *b,
            CnvValue::String(s) => !s.is_empty(), // TODO: check
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            CnvValue::Integer(i) => i.to_string(),
            CnvValue::Double(d) => d.to_string(), // TODO: check
            CnvValue::Boolean(b) => b.to_string(),  //TODO: check
            CnvValue::String(s) => s.clone(),
        }
    }
}

impl Add for &CnvValue {
    type Output = CnvValue;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i + rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d + rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b || rhs.to_boolean()),
            CnvValue::String(s) => CnvValue::String(s.clone() + rhs.to_string().as_ref()),
        }
    }
}

impl Mul for &CnvValue {
    type Output = CnvValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i * rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d * rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b && rhs.to_boolean()),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

impl Sub for &CnvValue {
    type Output = CnvValue;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i - rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d - rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b && !rhs.to_boolean()),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

impl Div for &CnvValue {
    type Output = CnvValue;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i / rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d / rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

impl Rem for &CnvValue {
    type Output = CnvValue;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i % rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d % rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

pub type CnvObjects = HashMap<String, Box<dyn CnvObject>>;

pub trait CnvExpression {
    fn calculate(&self, objects: &mut CnvObjects) -> Option<CnvValue>;
}

pub trait CnvStatement {
    fn run(&self, objects: &mut CnvObjects);
}

impl CnvExpression for Invocation {
    fn calculate(&self, objects: &mut CnvObjects) -> Option<CnvValue> {
        if self.parent.is_none() {
            match &self.name {
                _ => None,
            }
        } else {
            let _parent = self.parent.as_ref().unwrap().calculate(objects);
            match objects.get_mut("") { // TODO: stringify parent
                Some(obj) => obj.call_method(&self.name),
                None => None,  // error
            }
        }
    }
}

impl CnvExpression for Expression {
    fn calculate(&self, objects: &mut CnvObjects) -> Option<CnvValue> {
        match self {
            Expression::LiteralBool(b) => Some(CnvValue::Boolean(*b)),
            Expression::Identifier(name) => objects.get_mut(&name[..]).and_then(|x| x.get_value()), // error
            Expression::Parameter(_name) => None, // access function scope and retrieve arguments
            Expression::NameResolution(expression) => {
                let _name = &expression.calculate(objects);
                let name = String::new(); // TODO: stringify
                objects.get_mut(&name[..]).and_then(|x| x.get_value()) // error
            },
            Expression::FieldAccess(_expression, _field) => todo!(),
            Expression::Operation(expression, operations) => {
                let mut result = expression.calculate(objects).expect("Expected non-void argument in operation");
                for (operation, argument) in operations {
                    let argument = argument.calculate(objects).expect("Expected non-void argument in operation");
                    result = match operation {
                        Operation::Addition => &result + &argument,
                        Operation::Multiplication => &result * &argument,
                        Operation::Subtraction => &result - &argument,
                        Operation::Division => &result / &argument,
                        Operation::Remainder => &result % &argument,
                    }
                }
                Some(result)
            },
            Expression::Block(_block) => todo!(), // create a temporary function
        }
    }
}

impl CnvStatement for IgnorableProgram {
    fn run(&self, objects: &mut CnvObjects) {
        if self.ignored {
            return;
        }
        self.value.run(objects);
    }
}

impl CnvStatement for Program {
    fn run(&self, objects: &mut CnvObjects) {
        match self {
            Program::Identifier(identifier) => {
                let _obj = objects.get(identifier).unwrap_or_else(|| panic!("Expected existing object named {}", &identifier));
                todo!(); // run object
            },
            Program::Block(ignorable_statements) => {
                for ignorable_statement in ignorable_statements {
                    ignorable_statement.run(objects);
                }
            },
        }
    }
}

impl CnvStatement for IgnorableStatement {
    fn run(&self, objects: &mut CnvObjects) {
        if self.ignored {
            return;
        }
        self.value.run(objects);
    }
}

impl CnvStatement for Statement {
    fn run(&self, objects: &mut CnvObjects) {
        match self {
            Statement::Invocation(invocation) => {
                invocation.calculate(objects);
            },
            Statement::ExpressionStatement(expression) => {
                expression.calculate(objects);
            }
        }
    }
}

#[allow(dead_code)]
struct CnvRunner {
    objects: HashMap<String, Box<dyn CnvObject>>,
}

#[allow(dead_code)]
impl CnvRunner {
    pub fn run(&mut self, program: &IgnorableProgram) {
        program.run(&mut self.objects);
    }
}
