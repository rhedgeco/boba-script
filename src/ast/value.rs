use std::ops::{Add, Div, Mul, Neg, Sub};

use derive_more::Display;

use crate::token::Span;

use super::{BobaError, Color, ErrorLabel};

#[derive(Debug, Display, Clone, PartialEq, PartialOrd)]
pub enum Value {
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    Float(f64),
    #[display(fmt = "'{}'", _0)]
    String(String),
}

pub struct OpError {
    message: String,
}

impl OpError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn into_boba(self, span: Span) -> BobaError {
        BobaError {
            message: format!("Operator Error"),
            labels: vec![ErrorLabel {
                message: self.message,
                color: Color::Red,
                span,
            }],
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, OpError>;

    fn neg(self) -> Self::Output {
        use Value as V;
        match self {
            V::Int(v) => Ok(V::Int(-v)),
            V::Float(v) => Ok(V::Float(-v)),
            V::String(_) => Err(OpError::new("Cannot negate strings")),
        }
    }
}

impl Add<Value> for Value {
    type Output = Result<Value, OpError>;

    fn add(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => Ok(V::Float(v2 + v1 as f64)),
            (V::Int(v1), V::Int(v2)) => Ok(V::Int(v1 + v2)),
            (V::Float(v1), V::Float(v2)) => Ok(V::Float(v1 + v2)),
            (V::String(s), V::Int(v)) => Ok(V::String(format!("{s}{v}"))),
            (V::String(s), V::Float(v)) => Ok(V::String(format!("{s}{v}"))),
            (V::String(s1), V::String(s2)) => Ok(V::String(format!("{s1}{s2}"))),
            (V::Int(_), V::String(_)) => Err(OpError::new("Cannot add integer and string")),
            (V::Float(_), V::String(_)) => Err(OpError::new("Cannot add float and string")),
        }
    }
}

impl Sub<Value> for Value {
    type Output = Result<Value, OpError>;

    fn sub(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => Ok(V::Float(v2 - v1 as f64)),
            (V::Int(v1), V::Int(v2)) => Ok(V::Int(v1 - v2)),
            (V::Float(v1), V::Float(v2)) => Ok(V::Float(v1 - v2)),
            (V::String(_), V::Int(_)) => Err(OpError::new("Cannot subtract string and integer")),
            (V::String(_), V::Float(_)) => Err(OpError::new("Cannot subtract string and float")),
            (V::String(_), V::String(_)) => Err(OpError::new("Cannot subtract string and string")),
            (V::Int(_), V::String(_)) => Err(OpError::new("Cannot subtract integer and string")),
            (V::Float(_), V::String(_)) => Err(OpError::new("Cannot subtract float and string")),
        }
    }
}

impl Mul<Value> for Value {
    type Output = Result<Value, OpError>;

    fn mul(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => Ok(V::Float(v2 * v1 as f64)),
            (V::Int(v1), V::Int(v2)) => Ok(V::Int(v1 * v2)),
            (V::Float(v1), V::Float(v2)) => Ok(V::Float(v1 * v2)),
            (V::String(s), V::Int(v)) => Ok(V::String(s.repeat(v as usize))),
            (V::String(_), V::Float(_)) => Err(OpError::new("Cannot multiply string and float")),
            (V::String(_), V::String(_)) => Err(OpError::new("Cannot multiply string and string")),
            (V::Int(_), V::String(_)) => Err(OpError::new("Cannot multiply integer and string")),
            (V::Float(_), V::String(_)) => Err(OpError::new("Cannot multiply float and string")),
        }
    }
}

impl Div<Value> for Value {
    type Output = Result<Value, OpError>;

    fn div(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => Ok(V::Float(v2 / v1 as f64)),
            (V::Int(v1), V::Int(v2)) => Ok(V::Float(v1 as f64 / v2 as f64)),
            (V::Float(v1), V::Float(v2)) => Ok(V::Float(v1 / v2)),
            (V::String(_), V::Int(_)) => Err(OpError::new("Cannot divide string and integer")),
            (V::String(_), V::Float(_)) => Err(OpError::new("Cannot divide string and float")),
            (V::String(_), V::String(_)) => Err(OpError::new("Cannot divide string and string")),
            (V::Int(_), V::String(_)) => Err(OpError::new("Cannot divide integer and string")),
            (V::Float(_), V::String(_)) => Err(OpError::new("Cannot divide float and string")),
        }
    }
}

impl Value {
    pub fn pow(self, rhs: Value) -> Result<Value, OpError> {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) => Ok(V::Float((v1 as f64).powf(v2))),
            (V::Float(v1), V::Int(v2)) => Ok(V::Float((v1).powf(v2 as f64))),
            (V::Float(v1), V::Float(v2)) => Ok(V::Float(v1.powf(v2))),
            (V::Int(v1), V::Int(v2)) => Ok(V::Float((v1 as f64).powf(v2 as f64))),
            (V::String(_), V::Int(_)) => Err(OpError::new("Cannot ** on string and integer")),
            (V::String(_), V::Float(_)) => Err(OpError::new("Cannot ** on string and float")),
            (V::String(_), V::String(_)) => Err(OpError::new("Cannot ** on string and string")),
            (V::Int(_), V::String(_)) => Err(OpError::new("Cannot ** on integer and string")),
            (V::Float(_), V::String(_)) => Err(OpError::new("Cannot ** on float and string")),
        }
    }
}
