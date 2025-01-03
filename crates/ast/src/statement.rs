use crate::{Definition, Expr, Node, Pattern};

#[derive(Debug, Clone)]
pub enum Statement<Type> {
    Global(Node<Definition<Type>>),
    Local(LocalStatement<Type>),
}

#[derive(Debug, Clone)]
pub enum LocalStatement<Type> {
    DanglingExpr(Node<Expr<Type>>),
    Assignment(Node<Assignment<Type>>),
}

#[derive(Debug, Clone)]
pub struct Assignment<Type> {
    pub init: Node<bool>,
    pub pattern: Node<Pattern>,
    pub expr: Node<Expr<Type>>,
}
