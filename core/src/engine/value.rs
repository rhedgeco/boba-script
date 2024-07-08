use dashu::integer::IBig;
use derive_more::Display;

#[derive(Debug, Display, Clone, PartialEq)]
pub enum Value {
    #[display(fmt = "none")]
    None,
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "{}", _0)]
    Int(IBig),
    #[display(fmt = "{}", _0)]
    Float(f64),
    #[display(fmt = "'{}'", _0)]
    String(String),
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
        }
        .into()
    }
}
