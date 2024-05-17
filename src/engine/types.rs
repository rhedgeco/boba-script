use derive_more::Display;

#[derive(Debug, Display, Clone)]
pub enum Value {
    #[display(fmt = "unit")]
    Unit,
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    Float(f64),
    #[display(fmt = "'{}'", _0)]
    String(String),
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Unit => format!("unit"),
            Value::Bool(_) => format!("bool"),
            Value::Int(_) => format!("int"),
            Value::Float(_) => format!("float"),
            Value::String(_) => format!("string"),
        }
    }
}
