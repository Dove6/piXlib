use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Rem, Sub},
    sync::Arc,
};

use crate::classes::{CnvContent, CnvObject};

use super::{RunnerContext, RunnerResult};

#[derive(Debug, Clone)]
pub enum CnvValue {
    Integer(i32),
    Double(f64),
    Boolean(bool),
    String(String),
    Reference(Arc<CnvObject>),
}

impl Display for CnvValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CnvValue::Integer(i) => write!(f, "CnvValue::Integer({})", i),
            CnvValue::Double(d) => write!(f, "CnvValue::Double({})", d),
            CnvValue::Boolean(b) => write!(f, "CnvValue::Boolean({})", b),
            CnvValue::String(s) => write!(f, "CnvValue::String({})", &s),
            CnvValue::Reference(r) => write!(f, "CnvValue::Reference({})", &r.name),
        }
    }
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
            CnvValue::Reference(r) => get_reference_value(r).unwrap().unwrap().to_integer(),
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
            CnvValue::Reference(r) => get_reference_value(r).unwrap().unwrap().to_double(),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            CnvValue::Integer(i) => *i != 0,  // TODO: check
            CnvValue::Double(d) => *d != 0.0, // TODO: check
            CnvValue::Boolean(b) => *b,
            CnvValue::String(s) => !s.is_empty(), // TODO: check
            CnvValue::Reference(r) => get_reference_value(r).unwrap().unwrap().to_boolean(),
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            CnvValue::Integer(i) => i.to_string(),
            CnvValue::Double(d) => d.to_string(), // TODO: check
            CnvValue::Boolean(b) => b.to_string(), //TODO: check
            CnvValue::String(s) => s.clone(),
            CnvValue::Reference(r) => get_reference_value(r)
                .unwrap()
                .inspect(|v| eprintln!("Got reference value: {}", v))
                .unwrap_or(CnvValue::String(r.name.clone()))
                .to_string(),
            // r.name.clone(), // TODO: not always
        }
    }

    pub fn resolve(self, context: RunnerContext) -> CnvValue {
        match &self {
            CnvValue::String(s) => context
                .runner
                .get_object(&s)
                .inspect(|v| eprintln!("Resolving {:?} through {}", &self, v.name))
                .map(|o| {
                    let s = CnvValue::Reference(o.clone()).to_string();
                    if let Some(o) = context.runner.get_object(&s) {
                        CnvValue::Reference(o)
                    } else {
                        CnvValue::String(s)
                    }
                })
                .inspect(|v| eprintln!("Resolved into {}", v))
                .unwrap_or(CnvValue::String(trim_one_quotes_level(&s).to_owned())),
            CnvValue::Reference(r) => get_reference_value(r).unwrap().inspect(|v| eprintln!("Got reference value: {}", v)).unwrap_or(self),
            _ => self,
        }
    }
}

fn get_reference_value(r: &Arc<CnvObject>) -> RunnerResult<Option<CnvValue>> {
    let context = RunnerContext::new_minimal(&r.parent.runner, r);
    match &*r.content.borrow() {
        CnvContent::Expression(e) => Some(e.calculate()).transpose(),
        CnvContent::Behavior(b) => b.run_c(context, Vec::new()),
        CnvContent::Integer(i) => i.get().map(|v| Some(CnvValue::Integer(v))),
        CnvContent::Double(d) => d.get().map(|v| Some(CnvValue::Double(v))),
        CnvContent::Bool(b) => b.get().map(|v| Some(CnvValue::Boolean(v))),
        CnvContent::String(s) => s.get().map(|v| Some(CnvValue::String(v))),
        _ => Ok(None),
    }
}

fn trim_one_quotes_level(string: &str) -> &str {
    let start: usize = if string.starts_with('"') { 1 } else { 0 };
    let end: usize = string.len() - if string.ends_with('"') { 1 } else { 0 };
    &string[start..end]
}

impl Add for &CnvValue {
    type Output = CnvValue;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i + rhs.to_integer()),
            CnvValue::Double(d) => CnvValue::Double(*d + rhs.to_double()),
            CnvValue::Boolean(b) => CnvValue::Boolean(*b || rhs.to_boolean()),
            CnvValue::String(s) => CnvValue::String(s.clone() + rhs.to_string().as_ref()),
            CnvValue::Reference(r) => {
                if let Some(value) = get_reference_value(r).unwrap() {
                    &value + rhs
                } else {
                    todo!()
                }
            }
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
            CnvValue::Reference(r) => {
                if let Some(value) = get_reference_value(r).unwrap() {
                    &value * rhs
                } else {
                    todo!()
                }
            }
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
            CnvValue::Reference(r) => {
                if let Some(value) = get_reference_value(r).unwrap() {
                    &value * rhs
                } else {
                    todo!()
                }
            }
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
            CnvValue::Reference(r) => {
                if let Some(value) = get_reference_value(r).unwrap() {
                    &value / rhs
                } else {
                    todo!()
                }
            }
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
            CnvValue::Reference(r) => {
                if let Some(value) = get_reference_value(r).unwrap() {
                    &value % rhs
                } else {
                    todo!()
                }
            }
        }
    }
}

impl PartialEq for CnvValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            CnvValue::Integer(i) => *i == other.to_integer(),
            CnvValue::Double(d) => *d == other.to_double(),
            CnvValue::Boolean(b) => *b == other.to_boolean(),
            CnvValue::String(s) => *s == other.to_string(),
            CnvValue::Reference(r) => {
                if let Some(value) = get_reference_value(r).unwrap() {
                    &value == other
                } else {
                    todo!()
                }
            }
        }
    }
}
