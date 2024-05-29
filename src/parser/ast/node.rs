use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct Node<Item, Data> {
    item: Item,
    data: Data,
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

impl<Item, Data> Node<Item, Data> {
    pub fn new(item: Item, data: Data) -> Self {
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

    pub fn into_parts(self) -> (Item, Data) {
        (self.item, self.data)
    }
}
