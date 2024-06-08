use std::sync::Arc;

use crate::parser::ast::Func;

use super::Value;

#[derive(Debug, Clone)]
pub enum FuncValue<Data> {
    Custom(Arc<Func<Data>>),
    Native(Arc<NativeFunc<Data>>),
}

impl<Data> FuncValue<Data> {
    pub fn custom(func: Func<Data>) -> Self {
        Self::Custom(Arc::new(func))
    }

    pub fn native(func: NativeFunc<Data>) -> Self {
        Self::Native(Arc::new(func))
    }

    pub fn param_count(&self) -> usize {
        match self {
            FuncValue::Custom(func) => func.params.len(),
            FuncValue::Native(func) => func.param_count,
        }
    }
}

pub type NativeFuncImpl<Data> = fn(Vec<Value<Data>>) -> Result<Value<Data>, String>;

#[derive(Debug, Clone)]
pub struct NativeFunc<Data> {
    pub param_count: usize,
    pub native: NativeFuncImpl<Data>,
}

impl<Data> NativeFunc<Data> {
    pub fn new(param_count: usize, native: NativeFuncImpl<Data>) -> Self {
        Self {
            param_count,
            native,
        }
    }
}
