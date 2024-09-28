use boba_script_ast::{int::IBig, Func};
use derive_more::Display;

#[derive(Debug, Display, Clone, PartialEq, PartialOrd)]
pub enum Value {
    #[display("none")]
    None,
    #[display("{_0}")]
    Bool(bool),
    #[display("{_0}")]
    Int(IBig),
    #[display("{_0}")]
    Float(f64),
    #[display("{_0}")]
    Str(String),
    #[display("fn({})", _0.params.len())]
    Fn(Func),
}

impl Value {
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::None => ValueKind::None,
            Value::Bool(_) => ValueKind::Bool,
            Value::Int(_) => ValueKind::Int,
            Value::Float(_) => ValueKind::Float,
            Value::Str(_) => ValueKind::Str,
            Value::Fn(_) => ValueKind::Fn,
        }
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueKind {
    #[display("none")]
    None,
    #[display("bool")]
    Bool,
    #[display("int")]
    Int,
    #[display("float")]
    Float,
    #[display("str")]
    Str,
    #[display("fn")]
    Fn,
}
