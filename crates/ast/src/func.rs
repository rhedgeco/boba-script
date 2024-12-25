use crate::{path::PathUnion, statement::Statement, Node};

#[derive(Debug, Clone)]
pub struct Func {
    pub parameters: Vec<Node<FuncParam>>,
    pub output: Node<PathUnion>,
    pub body: Node<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct FuncParam {
    pub name: Node<String>,
    pub union: Node<PathUnion>,
}
