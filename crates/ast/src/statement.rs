use crate::{Definition, Expr, Node, Pattern};

#[derive(Debug)]
pub enum Statement {
    Global(Node<Definition>),
    Local(Node<LocalStatement>),
}

#[derive(Debug)]
pub enum LocalStatement {
    Let {
        pattern: Node<Pattern>,
        expr: Node<Expr>,
    },
    Set {
        pattern: Node<Pattern>,
        expr: Node<Expr>,
    },
    Expr(Node<Expr>),
}
