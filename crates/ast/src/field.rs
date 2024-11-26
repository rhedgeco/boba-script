use crate::{path::Union, Expr, Node};

pub struct Field {
    pub name: Node<String>,
    pub union: Node<Union>,
}

pub struct InitField {
    pub name: Node<String>,
    pub expr: Node<Expr>,
}
