use std::collections::HashMap;

use crate::ast::Ident;

use super::types::Value;

pub trait EngineScope {
    fn has_var(&self, ident: &Ident) -> bool;
    fn get_var(&self, ident: &Ident) -> Option<&Value>;
}

#[derive(Debug, Default)]
pub struct Scope {
    vars: HashMap<Ident, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_var(&mut self, ident: Ident, value: Value) {
        self.vars.insert(ident, value);
    }

    pub fn set_var(&mut self, ident: &Ident, new_value: Value) -> Option<Value> {
        match self.vars.get_mut(ident) {
            None => Some(new_value),
            Some(value) => {
                *value = new_value;
                None
            }
        }
    }
}

impl EngineScope for Scope {
    fn has_var(&self, ident: &Ident) -> bool {
        self.vars.contains_key(ident)
    }
    fn get_var(&self, ident: &Ident) -> Option<&Value> {
        self.vars.get(ident)
    }
}

pub struct ScopeGroup<'a, S: EngineScope, SP: EngineScope> {
    scope: &'a S,
    parent: &'a SP,
}

impl<'a, S: EngineScope, SP: EngineScope> ScopeGroup<'a, S, SP> {
    pub fn new(scope: &'a S, parent: &'a SP) -> Self {
        Self { scope, parent }
    }
}

impl<'a, S: EngineScope, SP: EngineScope> EngineScope for ScopeGroup<'a, S, SP> {
    fn has_var(&self, ident: &Ident) -> bool {
        match self.scope.has_var(ident) {
            false => self.parent.has_var(ident),
            true => true,
        }
    }

    fn get_var(&self, ident: &Ident) -> Option<&Value> {
        match self.scope.get_var(ident) {
            None => self.parent.get_var(ident),
            Some(value) => Some(value),
        }
    }
}
