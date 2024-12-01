use crate::{path::Union, Expr, Node};

#[derive(Debug)]
pub struct Field {
    pub name: Node<String>,
    pub union: Node<Union>,
}

#[derive(Debug)]
pub struct InitField {
    pub name: Node<String>,
    pub expr: Node<Expr>,
}
