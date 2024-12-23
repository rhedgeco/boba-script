use crate::{Definition, Expr, Node, Pattern};

#[derive(Debug, Clone)]
pub enum Statement {
    Global(Node<Definition>),
    Local(LocalStatement),
}

#[derive(Debug, Clone)]
pub enum LocalStatement {
    DanglingExpr(Node<Expr>),
    Assignment(Node<Assignment>),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub init: Node<bool>,
    pub pattern: Node<Pattern>,
    pub expr: Node<Expr>,
}
