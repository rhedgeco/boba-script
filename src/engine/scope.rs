use std::collections::HashMap;

use crate::ast::Ident;

use super::types::Value;

#[derive(Debug, Default)]
pub struct Scope {
    vars: HashMap<String, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_var(&self, ident: &Ident) -> bool {
        self.vars.contains_key(ident.as_str())
    }

    pub fn init_var(&mut self, ident: Ident, value: Value) {
        self.vars.insert(ident.as_str().into(), value);
    }

    pub fn get_var(&self, ident: &Ident) -> Option<&Value> {
        self.vars.get(ident.as_str())
    }

    pub fn get_var_mut(&mut self, ident: &Ident) -> Option<&mut Value> {
        self.vars.get_mut(ident.as_str())
    }
}
