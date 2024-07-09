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
    #[display(fmt = "{}", "print_values(_0)")]
    Tuple(Vec<Value>),
}

fn print_values(values: &[Value]) -> String {
    let values = values
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("({values})")
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::None => "none".into(),
            Value::Bool(_) => "bool".into(),
            Value::Int(_) => "int".into(),
            Value::Float(_) => "float".into(),
            Value::String(_) => "string".into(),
            Value::Tuple(values) => {
                let values = values
                    .iter()
                    .map(|v| v.type_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({values})")
            }
        }
    }
}
