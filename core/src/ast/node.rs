use std::ops::{Deref, DerefMut};

use crate::{
    engine::{EvalError, Value},
    Engine,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node<Item, Data> {
    pub item: Item,
    pub data: Data,
}

impl<Item, Data> DerefMut for Node<Item, Data> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<Item, Data> Deref for Node<Item, Data> {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<Item, Data> AsRef<Node<Item, Data>> for Node<Item, Data> {
    fn as_ref(&self) -> &Node<Item, Data> {
        self
    }
}

impl<Item, Data> Node<Item, Data> {
    pub fn new(item: Item, data: Data) -> Self {
        Self { item, data }
    }
}

pub trait EvalNode<Data: Clone>: Sized {
    fn eval_node(
        node: &Node<Self, Data>,
        engine: &mut Engine<Data>,
    ) -> Result<Value, EvalError<Data>>;
}

pub trait Builder<Data>: Sized {
    fn build_node(self, data: Data) -> Node<Self, Data>;
}

impl<Data, T: Sized> Builder<Data> for T {
    fn build_node(self, data: Data) -> Node<Self, Data> {
        Node::new(self, data)
    }
}
