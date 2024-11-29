use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

pub type BNode<T> = Box<Node<T>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(usize);

impl NodeId {
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node<T> {
    pub id: NodeId,
    pub item: T,
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
    pub fn build(item: T) -> Self {
        Self {
            id: NodeId::new(),
            item,
        }
    }
}
