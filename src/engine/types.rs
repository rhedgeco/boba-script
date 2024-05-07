use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    #[display(fmt = "null")]
    Null,
    #[display(fmt = "int")]
    Int,
    #[display(fmt = "float")]
    Float,
    #[display(fmt = "string")]
    String,
}

#[derive(Debug, Display, Clone, PartialEq, PartialOrd)]
pub enum Value {
    #[display(fmt = "null")]
    Null,
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    Float(f64),
    #[display(fmt = "'{}'", _0)]
    String(String),
}

impl Value {
    pub fn ty(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
        }
    }
}
