use std::ops::{Deref, DerefMut};

use crate::{
    engine::{EvalError, Value},
    Engine,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node<Item, Source> {
    pub item: Item,
    pub source: Source,
}

impl<Item, Source> DerefMut for Node<Item, Source> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<Item, Source> Deref for Node<Item, Source> {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<Item, Source> AsRef<Node<Item, Source>> for Node<Item, Source> {
    fn as_ref(&self) -> &Node<Item, Source> {
        self
    }
}

impl<Item, Source> Node<Item, Source> {
    pub fn new(item: Item, source: Source) -> Self {
        Self { item, source }
    }
}

pub trait EvalNode<Source: Clone>: Sized {
    fn eval_node(
        node: &Node<Self, Source>,
        engine: &mut Engine<Source>,
    ) -> Result<Value, EvalError<Source>>;
}

pub trait Builder<Source>: Sized {
    fn build_node(self, source: Source) -> Node<Self, Source>;
}

impl<Source, T: Sized> Builder<Source> for T {
    fn build_node(self, source: Source) -> Node<Self, Source> {
        Node::new(self, source)
    }
}
