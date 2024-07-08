use std::mem::replace;

use super::Value;

#[derive(Debug, Default)]
pub struct ShadowScope {
    scopes: Vec<Vec<(String, Value)>>,
}

impl ShadowScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn init(&mut self, id: impl Into<String>, value: Value) {
        let entry = (id.into(), value);
        match self.scopes.last_mut() {
            Some(scope) => scope.push(entry),
            None => self.scopes.push(vec![entry]),
        }
    }

    pub fn set(&mut self, id: impl AsRef<str>, value: Value) -> Result<Value, Value> {
        let id = id.as_ref();
        let Some((scope_index, value_index)) = self.find(id) else {
            return Err(value);
        };

        let entry = &mut self.scopes[scope_index][value_index];
        Ok(replace(entry, (id.to_string(), value)).1)
    }

    pub fn get(&self, id: impl AsRef<str>) -> Option<&Value> {
        let (scope_index, value_index) = self.find(id.as_ref())?;
        Some(&self.scopes[scope_index][value_index].1)
    }

    fn find(&self, id: &str) -> Option<(usize, usize)> {
        for (scope_index, scope) in self.scopes.iter().enumerate().rev() {
            for (value_index, (value_id, _)) in scope.iter().enumerate().rev() {
                if value_id.as_str() == id {
                    return Some((scope_index, value_index));
                }
            }
        }

        None
    }
}
