use crate::Node;

#[derive(Debug, Clone)]
pub enum Type {
    Any,
    None,
    Bool,
    Int,
    Float,
    String,
    Path(TypePath),
}

#[derive(Debug, Clone)]
pub struct TypePath {
    pub parts: Vec<Node<PathPart>>,
}

#[derive(Debug, Clone)]
pub enum PathPart {
    Ident(String),
    Super,
}

#[derive(Debug, Clone)]
pub struct PathUnion {
    pub types: Vec<Node<Type>>,
}
