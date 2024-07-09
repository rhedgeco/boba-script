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
    #[display(fmt = "({})", "print_values(_0)")]
    Tuple(Vec<Value>),
}

fn print_values(values: &[Value]) -> String {
    values
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Value {
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::None => ValueKind::None,
            Value::Bool(_) => ValueKind::Bool,
            Value::Int(_) => ValueKind::Int,
            Value::Float(_) => ValueKind::Float,
            Value::String(_) => ValueKind::String,
            Value::Tuple(values) => {
                ValueKind::Tuple(values.iter().map(|v| v.kind()).collect::<Vec<_>>())
            }
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
    #[display(fmt = "{}", "print_value_kinds(_0)")]
    Tuple(Vec<ValueKind>),
}

fn print_value_kinds(values: &[ValueKind]) -> String {
    values
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(", ")
}
