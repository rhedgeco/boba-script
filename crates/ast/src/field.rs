use crate::{Expr, Node, Union};

pub struct Field {
    pub name: Node<String>,
    pub ty: Node<Union>,
}

pub struct InitField {
    pub name: Node<String>,
    pub expr: Node<Expr>,
}
