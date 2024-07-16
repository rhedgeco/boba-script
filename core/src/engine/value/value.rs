use std::fmt;

use dashu::integer::IBig;
use derive_more::Display;

use super::{
    func::FuncKind,
    tuple::{Tuple, TupleKind},
    FuncPtr,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<Source> {
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Tuple(Tuple<Source>),
    Func(FuncPtr<Source>),
}

impl<Source> fmt::Display for Value<Source> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "none"),
            Value::Bool(v) => write!(f, "{v}",),
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Tuple(v) => write!(f, "{v}"),
            Value::Func(v) => write!(f, "{v}"),
        }
    }
}

impl<Source> Value<Source> {
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::None => ValueKind::None,
            Value::Bool(_) => ValueKind::Bool,
            Value::Int(_) => ValueKind::Int,
            Value::Float(_) => ValueKind::Float,
            Value::String(_) => ValueKind::String,
            Value::Func(v) => ValueKind::Func(v.kind()),
            Value::Tuple(v) => ValueKind::Tuple(v.kind()),
        }
    }
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum ValueKind {
    #[display(fmt = "none")]
    None,
    #[display(fmt = "bool")]
    Bool,
    #[display(fmt = "int")]
    Int,
    #[display(fmt = "float")]
    Float,
    #[display(fmt = "string")]
    String,
    #[display(fmt = "{}", _0)]
    Tuple(TupleKind),
    #[display(fmt = "{}", _0)]
    Func(FuncKind),
}
