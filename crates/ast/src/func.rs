use crate::{path::Union, statement::Statement, Field, Node};

pub struct Func {
    pub inputs: Vec<Node<Field>>,
    pub output: Node<Union>,
    pub body: Vec<Node<Statement>>,
}
