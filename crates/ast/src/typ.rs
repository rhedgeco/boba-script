use crate::Node;

#[derive(Debug, Clone)]
pub struct TypeUnion<Type> {
    pub types: Vec<Node<Type>>,
}

#[derive(Debug, Clone)]
pub enum PathPart {
    Ident(String),
    Super,
    Pearl,
}

#[derive(Debug, Clone)]
pub struct TypePath {
    pub path: Vec<Node<PathPart>>,
}
