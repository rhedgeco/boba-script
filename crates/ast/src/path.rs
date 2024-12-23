use crate::Node;

#[derive(Debug, Clone)]
pub enum TypePath {
    Any,
    None,
    Bool,
    Int,
    Float,
    String,
    Path(Vec<Node<PathPart>>),
}

#[derive(Debug, Clone)]
pub enum PathPart {
    Ident(String),
    Super,
}

#[derive(Debug, Clone)]
pub struct PathUnion {
    pub types: Vec<Node<TypePath>>,
}
