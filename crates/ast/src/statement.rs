use crate::{Definition, Expr, Node, Pattern};

pub enum Statement {
    Def(Node<Definition>),
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
