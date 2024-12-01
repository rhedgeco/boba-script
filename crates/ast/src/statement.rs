use crate::{Definition, Expr, Node, Pattern};

pub enum Statement {
    Global(Node<Definition>),
    Local(Node<LocalStatement>),
}

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
