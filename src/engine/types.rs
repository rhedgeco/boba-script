use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    #[display(fmt = "null")]
    Null,
    #[display(fmt = "int")]
    Int,
    #[display(fmt = "float")]
    Float,
}

#[derive(Debug, Display, Clone, PartialEq, PartialOrd)]
pub enum Value {
    #[display(fmt = "null")]
    Null,
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    Float(f64),
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl Value {
    pub fn ty(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
        }
    }
}
