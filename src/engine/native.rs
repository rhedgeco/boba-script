use crate::Engine;

use super::Value;

#[derive(Debug, Clone)]
pub struct NativeFunc {
    pub ident: String,
    pub params: Vec<String>,
    pub native: fn(&mut Engine) -> Result<Value, String>,
}

pub fn native_print() -> NativeFunc {
    NativeFunc {
        ident: format!("print"),
        params: vec![format!("message")],
        native: |engine| match engine.get_var("message") {
            None => panic!("message not found in function scope"),
            Some(value) => {
                println!("{value}");
                Ok(Value::None)
            }
        },
    }
}
