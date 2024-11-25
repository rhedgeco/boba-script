use crate::Node;

pub struct Union {
    pub types: Vec<Node<ConcreteType>>,
}

pub struct ConcreteType {
    pub path: Vec<Node<String>>,
}
