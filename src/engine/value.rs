use std::fmt::Display;

use dashu::{float::DBig, integer::IBig};

use super::FuncValue;

#[derive(Debug, Clone)]
pub enum Value<Data> {
    None,
    Bool(bool),
    Int(IBig),
    Float(DBig),
    String(String),
    Func(FuncValue<Data>),
}

#[derive(Debug, Clone)]
pub enum ValueType {
    None,
    Bool,
    Int,
    Float,
    String,
    Func(usize),
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::None => write!(f, "none"),
            ValueType::Bool => write!(f, "bool"),
            ValueType::Int => write!(f, "int"),
            ValueType::Float => write!(f, "float"),
            ValueType::String => write!(f, "string"),
            ValueType::Func(params) => write!(f, "fn({})", params),
        }
    }
}

impl<Data> Display for Value<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "none"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "'{v}'"),
            Value::Func(v) => write!(f, "fn({})", v.param_count()),
        }
    }
}

impl<Data> Value<Data> {
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::None => ValueType::None,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::Func(f) => ValueType::Func(f.param_count()),
        }
    }
}
