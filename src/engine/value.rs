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

impl<Data> Display for Value<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => write!(f, "none"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Func(v) => write!(f, "fn {}", v.ident()),
        }
    }
}

impl<Data> Value<Data> {
    pub fn type_name(&self) -> String {
        match self {
            Value::None => format!("none"),
            Value::Bool(_) => format!("bool"),
            Value::Int(_) => format!("int"),
            Value::Float(_) => format!("float"),
            Value::String(_) => format!("string"),
            Value::Func(_) => format!("function"),
        }
    }
}
