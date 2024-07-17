use std::mem::replace;

use super::Value;

enum StoreType {
    None,
    Global {
        scope_index: usize,
        value_index: usize,
    },
    Local {
        scope_index: usize,
        value_index: usize,
    },
}

pub struct ValueStore<Source> {
    globals: Vec<Vec<(String, Value<Source>)>>,
    locals: Vec<Vec<(String, Value<Source>)>>,
    stash: Vec<Vec<Vec<(String, Value<Source>)>>>,
}

impl<Source> Default for ValueStore<Source> {
    fn default() -> Self {
        Self {
            globals: Default::default(),
            locals: Default::default(),
            stash: Default::default(),
        }
    }
}

impl<Source> ValueStore<Source> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.locals.push(Vec::new());
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }

    pub fn stash(&mut self) {
        let values = replace(&mut self.locals, Vec::new());
        self.stash.push(values);
        self.push_scope();
    }

    pub fn unstash(&mut self) {
        let values = match self.stash.pop() {
            Some(stash) => stash,
            None => Vec::new(),
        };

        self.locals = values;
    }

    pub fn init_local(&mut self, id: impl Into<String>, value: Value<Source>) {
        let entry = (id.into(), value);
        match self.locals.last_mut() {
            Some(scope) => scope.push(entry),
            None => self.locals.push(vec![entry]),
        }
    }

    pub fn init_global(&mut self, id: impl Into<String>, value: Value<Source>) {
        let entry = (id.into(), value);
        match self.globals.last_mut() {
            Some(scope) => scope.push(entry),
            None => self.globals.push(vec![entry]),
        }
    }

    pub fn set(
        &mut self,
        id: impl AsRef<str>,
        value: Value<Source>,
    ) -> Result<Value<Source>, Value<Source>> {
        let id = id.as_ref();
        let entry = match self.find(id) {
            StoreType::None => return Err(value),
            StoreType::Global {
                scope_index,
                value_index,
            } => &mut self.globals[scope_index][value_index],
            StoreType::Local {
                scope_index,
                value_index,
            } => &mut self.locals[scope_index][value_index],
        };

        Ok(replace(entry, (id.to_string(), value)).1)
    }

    pub fn get(&self, id: impl AsRef<str>) -> Option<&Value<Source>> {
        match self.find(id.as_ref()) {
            StoreType::None => todo!(),
            StoreType::Global {
                scope_index,
                value_index,
            } => Some(&self.globals[scope_index][value_index].1),
            StoreType::Local {
                scope_index,
                value_index,
            } => Some(&self.locals[scope_index][value_index].1),
        }
    }

    fn find(&self, id: &str) -> StoreType {
        for (scope_index, scope) in self.locals.iter().enumerate().rev() {
            for (value_index, (value_id, _)) in scope.iter().enumerate().rev() {
                if value_id.as_str() == id {
                    return StoreType::Local {
                        scope_index,
                        value_index,
                    };
                }
            }
        }

        for (scope_index, scope) in self.globals.iter().enumerate().rev() {
            for (value_index, (value_id, _)) in scope.iter().enumerate().rev() {
                if value_id.as_str() == id {
                    return StoreType::Global {
                        scope_index,
                        value_index,
                    };
                }
            }
        }

        StoreType::None
    }
}
