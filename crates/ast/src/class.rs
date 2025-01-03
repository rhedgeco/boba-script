use crate::{node::NodeId, path::PathUnion, Definition, Expr, Node, Visibility};

#[derive(Debug, Clone)]
pub struct Class {
    pub native: Option<NodeId>,
    pub fields: Vec<Node<Field>>,
    pub defs: Vec<Node<Definition>>,
}

#[derive(Debug, Clone)]
pub struct ClassInit {
    pub name: Node<String>,
    pub fields: Vec<Node<FieldInit>>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub vis: Node<Visibility>,
    pub name: Node<String>,
    pub union: Node<PathUnion>,
}

#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: Node<String>,
    pub expr: Node<Expr>,
}
