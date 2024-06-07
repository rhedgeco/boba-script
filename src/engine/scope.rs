use std::{
    hash::Hash,
    mem::replace,
    ops::{Index, IndexMut},
};

use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle(usize);

pub struct ShadowStore<T> {
    values: IndexMap<String, Vec<T>>,
}

impl<V> Default for ShadowStore<V> {
    fn default() -> Self {
        Self {
            values: Default::default(),
        }
    }
}

impl<T> IndexMut<Handle> for ShadowStore<T> {
    fn index_mut(&mut self, handle: Handle) -> &mut Self::Output {
        self.values[handle.0].last_mut().expect("valid handle")
    }
}

impl<T> Index<Handle> for ShadowStore<T> {
    type Output = T;

    fn index(&self, handle: Handle) -> &Self::Output {
        &self.values[handle.0].last().expect("valid handle")
    }
}

impl<T> ShadowStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&T> {
        self.values.get(key.as_ref())?.last()
    }

    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut T> {
        self.values.get_mut(key.as_ref())?.last_mut()
    }

    pub fn drop_one(&mut self, handle: Handle) {
        if let Some((_, values)) = self.values.get_index_mut(handle.0) {
            drop(values.pop());
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: T) -> Handle {
        use indexmap::map::Entry as E;
        match self.values.entry(key.into()) {
            E::Occupied(e) => {
                let handle = Handle(e.index());
                e.into_mut().push(value);
                handle
            }
            E::Vacant(e) => {
                let handle = Handle(e.index());
                e.insert(vec![value]);
                handle
            }
        }
    }
}

pub struct Scope<T> {
    values: ShadowStore<T>,
    stash: Vec<ShadowStore<T>>,
    scopes: Vec<Vec<Handle>>,
}

impl<V> Default for Scope<V> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            stash: Default::default(),
            scopes: Default::default(),
        }
    }
}

impl<T> IndexMut<Handle> for Scope<T> {
    fn index_mut(&mut self, handle: Handle) -> &mut Self::Output {
        &mut self.values[handle]
    }
}

impl<T> Index<Handle> for Scope<T> {
    type Output = T;

    fn index(&self, handle: Handle) -> &Self::Output {
        &self.values[handle]
    }
}

impl<T> Scope<T> {
    pub fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    pub fn pop_scope(&mut self) {
        if let Some(handles) = self.scopes.pop() {
            for handle in handles {
                self.values.drop_one(handle);
            }
        }
    }

    pub fn stash(&mut self) {
        let stash_values = replace(&mut self.values, ShadowStore::new());
        self.stash.push(stash_values);
        self.push_scope();
    }

    pub fn unstash(&mut self) {
        self.pop_scope();
        if let Some(stash_values) = self.stash.pop() {
            self.values = stash_values;
        }
    }

    pub fn get(&self, ident: impl AsRef<str>) -> Option<&T> {
        self.values.get(ident)
    }

    pub fn set(&mut self, ident: impl AsRef<str>, value: T) -> Option<T> {
        let old_value = self.values.get_mut(ident)?;
        Some(replace(old_value, value))
    }

    pub fn init(&mut self, ident: impl Into<String>, value: T) {
        let handle = self.values.insert(ident.into(), value);
        let Some(scope) = self.scopes.last_mut() else {
            let scope = vec![handle];
            self.scopes.push(scope);
            return;
        };

        scope.push(handle)
    }
}
