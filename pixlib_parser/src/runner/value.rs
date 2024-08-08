use std::{
    ops::{Add, Div, Mul, Rem, Sub},
    sync::Arc,
};

use crate::classes::{CallableIdentifier, CnvContent, CnvObject};

use super::RunnerResult;

#[derive(Debug, Clone)]
pub enum CnvValue {
    Integer(i32),
    Double(f64),
    Boolean(bool),
    String(String),
    Reference(Arc<CnvObject>),
}

impl CnvValue {
    pub fn to_integer(&self) -> i32 {
        match self {
            CnvValue::Integer(i) => *i,
            CnvValue::Double(d) => *d as i32,
            CnvValue::Boolean(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            CnvValue::String(s) => s.parse().unwrap(),
            CnvValue::Reference(r) => get_reference_value(r).unwrap().to_integer(),
        }
    }

    pub fn to_double(&self) -> f64 {
        match self {
            CnvValue::Integer(i) => (*i).into(),
            CnvValue::Double(d) => *d,
            CnvValue::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            CnvValue::String(s) => s
                .parse()
                // .inspect_err(|e| eprintln!("{} for string->double {}", e, s))
                .unwrap(),
            CnvValue::Reference(r) => get_reference_value(r).unwrap().to_double(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            CnvValue::Integer(i) => *i != 0,  // TODO: check
            CnvValue::Double(d) => *d != 0.0, // TODO: check
            CnvValue::Boolean(b) => *b,
            CnvValue::String(s) => !s.is_empty(), // TODO: check
            CnvValue::Reference(r) => get_reference_value(r).unwrap().to_boolean(),
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            CnvValue::Integer(i) => i.to_string(),
            CnvValue::Double(d) => d.to_string(), // TODO: check
            CnvValue::Boolean(b) => b.to_string(), //TODO: check
            CnvValue::String(s) => s.clone(),
            CnvValue::Reference(r) => r.name.clone(), // TODO: not always
        }
    }
}

fn get_reference_value(r: &Arc<CnvObject>) -> RunnerResult<CnvValue> {
    match &*r.content.borrow() {
        CnvContent::Expression(e) => e.calculate(),
        CnvContent::Integer(i) => i.get().map(|v| CnvValue::Integer(v)),
        CnvContent::Double(d) => d.get().map(|v| CnvValue::Double(v)),
        CnvContent::Bool(b) => b.get().map(|v| CnvValue::Boolean(v)),
        CnvContent::String(s) => s.get().map(|v| CnvValue::String(v)),
        _ => todo!(),
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
            CnvValue::Reference(_) => todo!(),
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
            CnvValue::Reference(_) => todo!(),
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
            CnvValue::Reference(_) => todo!(),
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
            CnvValue::Reference(_) => todo!(),
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
            CnvValue::Reference(_) => todo!(),
        }
    }
}

impl PartialEq for &CnvValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            CnvValue::Integer(i) => *i == other.to_integer(),
            CnvValue::Double(d) => *d == other.to_double(),
            CnvValue::Boolean(b) => *b == other.to_boolean(),
            CnvValue::String(s) => *s == other.to_string(),
            CnvValue::Reference(r) => {
                &r.call_method(CallableIdentifier::Method("GET"), &Vec::new(), None)
                    .unwrap()
                    .unwrap()
                    == other
            }
        }
    }
}
