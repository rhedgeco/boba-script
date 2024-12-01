use crate::{path::Union, statement::Statement, Field, Node};

#[derive(Debug, Clone)]
pub struct Func {
    pub inputs: Vec<Node<Field>>,
    pub output: Node<Union>,
    pub body: BodyKind,
}

#[derive(Debug, Clone)]
pub enum BodyKind {
    Script(Vec<Statement>),
    Native,
}
