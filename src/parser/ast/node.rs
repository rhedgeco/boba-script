use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct Node<Data, Item> {
    item: Item,
    data: Data,
}

impl<Data, Item> DerefMut for Node<Data, Item> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<Data, Item> Deref for Node<Data, Item> {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<Data, Item> Node<Data, Item> {
    pub fn new(data: Data, item: Item) -> Self {
        Self { item, data }
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn into_item(self) -> Item {
        self.item
    }

    pub fn into_data(self) -> Data {
        self.data
    }

    pub fn into_parts(self) -> (Data, Item) {
        (self.data, self.item)
    }
}
