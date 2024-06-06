use super::Value;

pub type NativeFuncImpl<Data> = fn(Vec<Value<Data>>) -> Result<Value<Data>, String>;

#[derive(Debug, Clone)]
pub struct NativeFunc<Data> {
    pub name: String,
    pub param_count: usize,
    pub native: NativeFuncImpl<Data>,
}

impl<Data> NativeFunc<Data> {
    pub fn new(name: impl Into<String>, param_count: usize, native: NativeFuncImpl<Data>) -> Self {
        Self {
            name: name.into(),
            param_count,
            native,
        }
    }
}
