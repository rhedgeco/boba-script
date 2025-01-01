use std::{
    any::TypeId,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use boba_script_ast::int::IBig;

use crate::indexers::ClassIndex;

#[derive(Debug, Clone)]
pub enum StackValue {
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Class(ClassLink),
}

pub trait Store {
    fn get(&self, key: ClassLink) -> Option<&ClassBox>;
    fn get_mut(&mut self, key: ClassLink) -> Option<&mut ClassBox>;
    fn store(&mut self, class: ClassBox) -> ClassLink;
    fn collect_garbage(&mut self);
}

#[derive(Debug, Default)]
pub struct ClassStore {
    classes: Vec<(ClassLink, ClassBox)>,
}

impl ClassStore {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Store for ClassStore {
    fn get(&self, key: ClassLink) -> Option<&ClassBox> {
        self.classes
            .get(key.0.load(Ordering::Relaxed))
            .map(|(_, c)| c)
    }

    fn get_mut(&mut self, key: ClassLink) -> Option<&mut ClassBox> {
        self.classes
            .get_mut(key.0.load(Ordering::Relaxed))
            .map(|(_, c)| c)
    }

    fn store(&mut self, class: ClassBox) -> ClassLink {
        let next_index = self.classes.len();
        let link = ClassLink(Arc::new(AtomicUsize::new(next_index)));
        self.classes.push((link.clone(), class));
        link
    }

    fn collect_garbage(&mut self) {
        todo!("collect garbage")
    }
}

#[derive(Debug, Clone)]
pub struct ClassLink(Arc<AtomicUsize>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClassKind {
    Native(TypeId),
    Script(ClassIndex),
}

#[derive(Debug)]
pub struct ClassBox {
    kind: ClassKind,
    ptr: *mut (),
}

impl ClassBox {
    pub fn kind(&self) -> ClassKind {
        self.kind
    }

    pub fn ptr(&mut self) -> *mut () {
        self.ptr
    }

    pub fn build<T: 'static>(data: T) -> Self {
        Self {
            kind: ClassKind::Native(TypeId::of::<T>()),
            ptr: Box::leak(Box::new(data)) as *mut T as *mut (),
        }
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        match &self.kind {
            ClassKind::Native(id) if id == &TypeId::of::<T>() => {
                Some(unsafe { &*(self.ptr as *mut T) })
            }
            _ => None,
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        match &self.kind {
            ClassKind::Native(id) if id == &TypeId::of::<T>() => {
                Some(unsafe { &mut *(self.ptr as *mut T) })
            }
            _ => None,
        }
    }

    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        match &self.kind {
            ClassKind::Native(id) if id == &TypeId::of::<T>() => {
                Ok(unsafe { *Box::from_raw(self.ptr as *mut T) })
            }
            _ => Err(self),
        }
    }
}
