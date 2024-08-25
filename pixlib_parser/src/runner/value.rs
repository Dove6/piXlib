use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Rem, Sub},
    sync::Arc,
};

use crate::runner::{content::CnvContent, CnvObject};

use super::RunnerContext;

#[derive(Debug, Clone)]
pub enum CnvValue {
    Integer(i32),
    Double(f64),
    Bool(bool),
    String(String),
}

impl Display for CnvValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CnvValue::Integer(i) => write!(f, "CnvValue::Integer({})", i),
            CnvValue::Double(d) => write!(f, "CnvValue::Double({})", d),
            CnvValue::Bool(b) => write!(f, "CnvValue::Bool({})", b),
            CnvValue::String(s) => write!(f, "CnvValue::String({})", &s),
        }
    }
}

impl CnvValue {
    pub fn to_int(&self) -> i32 {
        match self {
            CnvValue::Integer(i) => *i,
            CnvValue::Double(d) => *d as i32,
            CnvValue::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            CnvValue::String(s) => s.parse().unwrap(),
        }
    }

    pub fn to_dbl(&self) -> f64 {
        match self {
            CnvValue::Integer(i) => (*i).into(),
            CnvValue::Double(d) => *d,
            CnvValue::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            CnvValue::String(s) => s
                .parse()
                .inspect_err(|e| eprintln!("{} for string->double {}", e, s))
                .unwrap(),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            CnvValue::Integer(i) => *i != 0,  // TODO: check
            CnvValue::Double(d) => *d != 0.0, // TODO: check
            CnvValue::Bool(b) => *b,
            CnvValue::String(s) => !s.is_empty(), // TODO: check
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            CnvValue::Integer(i) => i.to_string(),
            CnvValue::Double(d) => d.to_string(), // TODO: check
            CnvValue::Bool(b) => b.to_string(),   //TODO: check
            CnvValue::String(s) => s.clone(),
        }
    }

    pub fn resolve(self, context: RunnerContext) -> CnvValue {
        match &self {
            CnvValue::String(s) => context
                .runner
                .get_object(s)
                // .inspect(|v| eprintln!("Resolving {:?} through {}", &self, v.name))
                .as_ref()
                .map(get_reference_value)
                .transpose()
                .unwrap()
                .flatten()
                // .inspect(|v| eprintln!("Resolved into {}", v))
                .unwrap_or(CnvValue::String(trim_one_quotes_level(s).to_owned())),
            _ => self,
        }
    }
}

fn get_reference_value(r: &Arc<CnvObject>) -> anyhow::Result<Option<CnvValue>> {
    let context = RunnerContext::new_minimal(&r.parent.runner, r);
    match &r.content {
        CnvContent::Expression(e) => Some(e.calculate()).transpose(),
        CnvContent::Behavior(b) => b.run_c(context, Vec::new()),
        CnvContent::Integer(i) => i.get().map(|v| Some(CnvValue::Integer(v))),
        CnvContent::Double(d) => d.get().map(|v| Some(CnvValue::Double(v))),
        CnvContent::Bool(b) => b.get().map(|v| Some(CnvValue::Bool(v))),
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
            CnvValue::Integer(i) => CnvValue::Integer(*i + rhs.to_int()),
            CnvValue::Double(d) => CnvValue::Double(*d + rhs.to_dbl()),
            CnvValue::Bool(b) => CnvValue::Bool(*b || rhs.to_bool()),
            CnvValue::String(s) => CnvValue::String(s.clone() + rhs.to_str().as_ref()),
        }
    }
}

impl Mul for &CnvValue {
    type Output = CnvValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i * rhs.to_int()),
            CnvValue::Double(d) => CnvValue::Double(*d * rhs.to_dbl()),
            CnvValue::Bool(b) => CnvValue::Bool(*b && rhs.to_bool()),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

impl Sub for &CnvValue {
    type Output = CnvValue;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i - rhs.to_int()),
            CnvValue::Double(d) => CnvValue::Double(*d - rhs.to_dbl()),
            CnvValue::Bool(b) => CnvValue::Bool(*b && !rhs.to_bool()),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

impl Div for &CnvValue {
    type Output = CnvValue;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i / rhs.to_int()),
            CnvValue::Double(d) => CnvValue::Double(*d / rhs.to_dbl()),
            CnvValue::Bool(b) => CnvValue::Bool(*b),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

impl Rem for &CnvValue {
    type Output = CnvValue;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            CnvValue::Integer(i) => CnvValue::Integer(*i % rhs.to_int()),
            CnvValue::Double(d) => CnvValue::Double(*d % rhs.to_dbl()),
            CnvValue::Bool(b) => CnvValue::Bool(*b),
            CnvValue::String(s) => CnvValue::String(s.clone()),
        }
    }
}

impl PartialEq for CnvValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            CnvValue::Integer(i) => *i == other.to_int(),
            CnvValue::Double(d) => *d == other.to_dbl(),
            CnvValue::Bool(b) => *b == other.to_bool(),
            CnvValue::String(s) => *s == other.to_str(),
        }
    }
}
