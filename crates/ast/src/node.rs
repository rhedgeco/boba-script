use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

pub type BNode<T> = Box<Node<T>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node<T> {
    id: NodeId,
    item: T,
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T> Node<T> {
    pub fn build(item: T) -> Node<T> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self {
            id: NodeId(COUNTER.fetch_add(1, Ordering::Relaxed)),
            item,
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}
