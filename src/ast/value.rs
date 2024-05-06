use std::ops::{Add, Div, Mul, Neg, Sub};

use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    Float(f64),
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        use Value as V;
        match self {
            V::Int(v) => V::Int(-v),
            V::Float(v) => V::Float(-v),
        }
    }
}

impl Add<Value> for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => V::Float(v2 + v1 as f64),
            (V::Int(v1), V::Int(v2)) => V::Int(v1 + v2),
            (V::Float(v1), V::Float(v2)) => V::Float(v1 + v2),
        }
    }
}

impl Sub<Value> for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => V::Float(v2 - v1 as f64),
            (V::Int(v1), V::Int(v2)) => V::Int(v1 - v2),
            (V::Float(v1), V::Float(v2)) => V::Float(v1 - v2),
        }
    }
}

impl Mul<Value> for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => V::Float(v2 * v1 as f64),
            (V::Int(v1), V::Int(v2)) => V::Int(v1 * v2),
            (V::Float(v1), V::Float(v2)) => V::Float(v1 * v2),
        }
    }
}

impl Div<Value> for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Self::Output {
        use Value as V;
        match (self, rhs) {
            (V::Int(v1), V::Float(v2)) | (V::Float(v2), V::Int(v1)) => V::Float(v2 / v1 as f64),
            (V::Int(v1), V::Int(v2)) => V::Float(v1 as f64 / v2 as f64),
            (V::Float(v1), V::Float(v2)) => V::Float(v1 / v2),
        }
    }
}
