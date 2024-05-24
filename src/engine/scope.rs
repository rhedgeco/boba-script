use std::{collections::HashMap, ops::Deref};

use crate::parser::ast::Func;

use super::{NativeFunc, Value};

#[derive(Debug, Clone)]
pub enum FuncType {
    Custom(Func),
    Native(NativeFunc),
}

impl FuncType {
    pub fn param_count(&self) -> usize {
        match self {
            FuncType::Custom(func) => func.params.len(),
            FuncType::Native(func) => func.param_count,
        }
    }
}

#[derive(Debug, Default)]
pub struct Scope {
    vars: HashMap<String, Value>,
    funcs: HashMap<String, FuncType>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_var(&self, ident: impl AsRef<str>) -> bool {
        self.vars.contains_key(ident.as_ref())
    }

    pub fn has_func(&self, ident: impl AsRef<str>) -> bool {
        self.funcs.contains_key(ident.as_ref())
    }

    pub fn init_var(&mut self, ident: impl Into<String>, value: Value) {
        self.vars.insert(ident.into(), value);
    }

    pub fn init_func(&mut self, func: Func) {
        self.funcs
            .insert(func.ident.deref().clone(), FuncType::Custom(func));
    }

    pub fn init_native_func(&mut self, func: NativeFunc) {
        self.funcs.insert(func.name.clone(), FuncType::Native(func));
    }

    pub fn get_var(&self, ident: impl AsRef<str>) -> Option<&Value> {
        self.vars.get(ident.as_ref())
    }

    pub fn get_func(&self, ident: impl AsRef<str>) -> Option<&FuncType> {
        self.funcs.get(ident.as_ref())
    }

    pub fn get_var_mut(&mut self, ident: impl AsRef<str>) -> Option<&mut Value> {
        self.vars.get_mut(ident.as_ref())
    }
}
