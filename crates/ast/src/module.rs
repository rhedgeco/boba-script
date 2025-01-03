use crate::{Definition, Node};

#[derive(Debug, Clone)]
pub struct Module<Type> {
    pub defs: Vec<Node<Definition<Type>>>,
}
