use derive_more::Display;

#[derive(Debug, Display, Clone)]
pub enum Value {
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "{}", _0)]
    Int(i64),
    #[display(fmt = "{}", _0)]
    Float(f64),
    #[display(fmt = "'{}'", _0)]
    String(String),
    #[display(fmt = "{} = {}", _0, _1)]
    Assignment(String, Box<Value>),
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Bool(_) => format!("bool"),
            Value::Int(_) => format!("int"),
            Value::Float(_) => format!("float"),
            Value::String(_) => format!("string"),
            Value::Assignment(_, _) => format!("assignment"),
        }
    }
}
