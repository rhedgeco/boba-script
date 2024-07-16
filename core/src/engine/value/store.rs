use std::mem::replace;

use super::Value;

pub struct ValueStore<Source> {
    values: Vec<Vec<(String, Value<Source>)>>,
}

impl<Source> Default for ValueStore<Source> {
    fn default() -> Self {
        Self {
            values: Default::default(),
        }
    }
}

impl<Source> ValueStore<Source> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.values.push(Vec::new());
    }

    pub fn pop_scope(&mut self) {
        self.values.pop();
    }

    pub fn init(&mut self, id: impl Into<String>, value: Value<Source>) {
        let entry = (id.into(), value);
        match self.values.last_mut() {
            Some(scope) => scope.push(entry),
            None => self.values.push(vec![entry]),
        }
    }

    pub fn set(
        &mut self,
        id: impl AsRef<str>,
        value: Value<Source>,
    ) -> Result<Value<Source>, Value<Source>> {
        let id = id.as_ref();
        let Some((scope_index, value_index)) = self.find(id) else {
            return Err(value);
        };

        let entry = &mut self.values[scope_index][value_index];
        Ok(replace(entry, (id.to_string(), value)).1)
    }

    pub fn get(&self, id: impl AsRef<str>) -> Option<&Value<Source>> {
        let (scope_index, value_index) = self.find(id.as_ref())?;
        Some(&self.values[scope_index][value_index].1)
    }

    fn find(&self, id: &str) -> Option<(usize, usize)> {
        for (scope_index, scope) in self.values.iter().enumerate().rev() {
            for (value_index, (value_id, _)) in scope.iter().enumerate().rev() {
                if value_id.as_str() == id {
                    return Some((scope_index, value_index));
                }
            }
        }

        None
    }
}
