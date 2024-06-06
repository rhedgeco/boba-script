use std::fmt::Display;

use dashu::{float::DBig, integer::IBig};

use crate::parser::ast::Func;

use super::NativeFunc;

#[derive(Debug, Clone)]
pub enum FuncValue<Data> {
    Custom(Func<Data>),
    Native(NativeFunc<Data>),
}

impl<Data> FuncValue<Data> {
    pub fn name(&self) -> &str {
        match self {
            FuncValue::Custom(func) => &*func.ident,
            FuncValue::Native(func) => &func.name,
        }
    }

    pub fn param_count(&self) -> usize {
        match self {
            FuncValue::Custom(func) => func.params.len(),
            FuncValue::Native(func) => func.param_count,
        }
    }
}

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
            Value::Func(v) => write!(f, "fn {}", v.name()),
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
