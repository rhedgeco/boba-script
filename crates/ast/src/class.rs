use crate::{node::NodeId, typ::TypeUnion, Definition, Expr, Node, Visibility};

#[derive(Debug, Clone)]
pub struct Class<Type> {
    pub native: Option<NodeId>,
    pub fields: Vec<Node<Field<Type>>>,
    pub defs: Vec<Node<Definition<Type>>>,
}

#[derive(Debug, Clone)]
pub struct ClassInit<Type> {
    pub name: Node<Type>,
    pub fields: Vec<Node<FieldInit<Type>>>,
}

#[derive(Debug, Clone)]
pub struct Field<Type> {
    pub vis: Node<Visibility>,
    pub name: Node<String>,
    pub union: Node<TypeUnion<Type>>,
}

#[derive(Debug, Clone)]
pub struct FieldInit<Type> {
    pub name: Node<String>,
    pub expr: Node<Expr<Type>>,
}
