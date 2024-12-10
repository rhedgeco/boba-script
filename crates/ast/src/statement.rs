use crate::{Definition, Expr, Node, Pattern};

#[derive(Debug, Clone)]
pub enum Statement {
    Global(Node<Definition>),
    Local(LocalStatement),
}

#[derive(Debug, Clone)]
pub enum LocalStatement {
    Let(Node<Assignment>),
    Set(Node<Assignment>),
    Expr(Node<Expr>),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub pattern: Node<Pattern>,
    pub expr: Node<Expr>,
}
