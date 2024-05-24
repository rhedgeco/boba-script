use super::Value;

pub type NativeFuncImpl = fn(Vec<Value>) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub struct NativeFunc {
    pub name: String,
    pub param_count: usize,
    pub native: NativeFuncImpl,
}

impl NativeFunc {
    pub fn new(name: impl Into<String>, param_count: usize, native: NativeFuncImpl) -> Self {
        Self {
            name: name.into(),
            param_count,
            native,
        }
    }
}

pub fn native_print() -> NativeFunc {
    NativeFunc::new("print", 1, |values| {
        let message = &values[0];
        println!("{message}");
        Ok(Value::None)
    })
}
