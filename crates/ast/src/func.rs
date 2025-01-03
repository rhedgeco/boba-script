use crate::{statement::Statement, typ::TypeUnion, Node};

#[derive(Debug, Clone)]
pub struct Func<Type> {
    pub parameters: Vec<Node<FuncParam<Type>>>,
    pub output: Node<TypeUnion<Type>>,
    pub body: Node<Vec<Statement<Type>>>,
}

#[derive(Debug, Clone)]
pub struct FuncParam<Type> {
    pub name: Node<String>,
    pub union: Node<TypeUnion<Type>>,
}
