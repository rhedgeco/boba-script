use crate::{statement::Statement, Field, Node, Union};

pub struct Func {
    pub inputs: Vec<Node<Field>>,
    pub output: Node<Union>,
    pub body: Vec<Node<Statement>>,
}
