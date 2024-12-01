use crate::{Definition, Node};

#[derive(Debug)]
pub struct Module {
    pub defs: Vec<Node<Definition>>,
}
