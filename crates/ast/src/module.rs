use crate::{Definition, Node};

#[derive(Debug, Clone)]
pub struct Module {
    pub defs: Vec<Node<Definition>>,
}
