use crate::Value;

pub trait Scope {
    fn get(&self, id: impl AsRef<str>) -> Option<&Value>;
    fn get_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value>;
    fn get_local(&self, id: impl AsRef<str>) -> Option<&Value>;
    fn get_local_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value>;
    fn get_global(&self, id: impl AsRef<str>) -> Option<&Value>;
    fn get_global_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value>;
    fn init_local(&mut self, id: impl Into<String>, value: Value);
    fn init_global(&mut self, id: impl Into<String>, value: Value);
}

impl<T: Scope> Scope for &mut T {
    fn get(&self, id: impl AsRef<str>) -> Option<&Value> {
        T::get(self, id)
    }

    fn get_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        T::get_mut(self, id)
    }

    fn get_local(&self, id: impl AsRef<str>) -> Option<&Value> {
        T::get_local(self, id)
    }

    fn get_local_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        T::get_local_mut(self, id)
    }

    fn get_global(&self, id: impl AsRef<str>) -> Option<&Value> {
        T::get_global(self, id)
    }

    fn get_global_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        T::get_global_mut(self, id)
    }

    fn init_local(&mut self, id: impl Into<String>, value: Value) {
        T::init_local(self, id, value)
    }

    fn init_global(&mut self, id: impl Into<String>, value: Value) {
        T::init_global(self, id, value)
    }
}

impl<T: Scope> ScopeExt for T {}
pub trait ScopeExt: Scope {
    fn nested(&mut self) -> NestedScope<&mut Self>
    where
        Self: Sized,
    {
        NestedScope {
            parent: self,
            child: LocalScope::new(),
        }
    }

    fn isolated(&mut self) -> IsolatedScope<&mut Self>
    where
        Self: Sized,
    {
        IsolatedScope {
            parent: self,
            child: LocalScope::new(),
        }
    }
}

struct ValueStore {
    global: bool,
    name: String,
    value: Value,
}

#[derive(Default)]
pub struct LocalScope {
    values: Vec<ValueStore>,
}

impl LocalScope {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Scope for LocalScope {
    fn get(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        for store in self.values.iter().rev() {
            if &store.name == id {
                return Some(&store.value);
            }
        }
        None
    }

    fn get_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        for store in self.values.iter_mut().rev() {
            if &store.name == id {
                return Some(&mut store.value);
            }
        }
        None
    }

    fn get_local(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        for store in self.values.iter().rev() {
            if !store.global && &store.name == id {
                return Some(&store.value);
            }
        }
        None
    }

    fn get_local_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        for store in self.values.iter_mut().rev() {
            if !store.global && &store.name == id {
                return Some(&mut store.value);
            }
        }
        None
    }

    fn get_global(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        for store in self.values.iter().rev() {
            if store.global && &store.name == id {
                return Some(&store.value);
            }
        }
        None
    }

    fn get_global_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        for store in self.values.iter_mut().rev() {
            if store.global && &store.name == id {
                return Some(&mut store.value);
            }
        }
        None
    }

    fn init_local(&mut self, id: impl Into<String>, value: Value) {
        self.values.push(ValueStore {
            global: false,
            name: id.into(),
            value,
        })
    }

    fn init_global(&mut self, id: impl Into<String>, value: Value) {
        self.values.push(ValueStore {
            global: true,
            name: id.into(),
            value,
        })
    }
}

pub struct NestedScope<Nested: Scope> {
    child: LocalScope,
    parent: Nested,
}

impl<Nested: Scope> Scope for NestedScope<Nested> {
    fn get(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        match self.child.get(id) {
            Some(value) => Some(value),
            None => self.parent.get(id),
        }
    }

    fn get_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        match self.child.get_mut(id) {
            Some(value) => Some(value),
            None => self.parent.get_mut(id),
        }
    }

    fn get_local(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        match self.child.get_local(id) {
            Some(value) => Some(value),
            None => self.parent.get_local(id),
        }
    }

    fn get_local_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        match self.child.get_local_mut(id) {
            Some(value) => Some(value),
            None => self.parent.get_local_mut(id),
        }
    }

    fn get_global(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        match self.child.get_global(id) {
            Some(value) => Some(value),
            None => self.parent.get_global(id),
        }
    }

    fn get_global_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        match self.child.get_global_mut(id) {
            Some(value) => Some(value),
            None => self.parent.get_global_mut(id),
        }
    }

    fn init_local(&mut self, id: impl Into<String>, value: Value) {
        self.child.init_local(id, value);
    }

    fn init_global(&mut self, id: impl Into<String>, value: Value) {
        self.child.init_global(id, value)
    }
}

pub struct IsolatedScope<Nested: Scope> {
    child: LocalScope,
    parent: Nested,
}

impl<Nested: Scope> Scope for IsolatedScope<Nested> {
    fn get(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        match self.child.get(id) {
            Some(value) => Some(value),
            None => self.parent.get_global(id),
        }
    }

    fn get_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        match self.child.get_mut(id) {
            Some(value) => Some(value),
            None => self.parent.get_global_mut(id),
        }
    }

    fn get_local(&self, id: impl AsRef<str>) -> Option<&Value> {
        self.child.get_local(id)
    }

    fn get_local_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        self.child.get_local_mut(id)
    }

    fn get_global(&self, id: impl AsRef<str>) -> Option<&Value> {
        let id = id.as_ref();
        match self.child.get_global(id) {
            Some(value) => Some(value),
            None => self.parent.get_global(id),
        }
    }

    fn get_global_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Value> {
        let id = id.as_ref();
        match self.child.get_global_mut(id) {
            Some(value) => Some(value),
            None => self.parent.get_global_mut(id),
        }
    }

    fn init_local(&mut self, id: impl Into<String>, value: Value) {
        self.child.init_local(id, value)
    }

    fn init_global(&mut self, id: impl Into<String>, value: Value) {
        self.child.init_global(id, value)
    }
}
