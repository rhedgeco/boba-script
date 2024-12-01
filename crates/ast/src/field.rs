use crate::{path::Union, Expr, Node};

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Node<String>,
    pub union: Node<Union>,
}

#[derive(Debug, Clone)]
pub struct InitField {
    pub name: Node<String>,
    pub expr: Node<Expr>,
}
