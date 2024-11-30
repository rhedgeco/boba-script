use crate::Node;

#[derive(Debug, Clone)]
pub struct Union {
    pub types: Vec<Node<ConcreteType>>,
}

#[derive(Debug, Clone)]
pub enum ConcreteType {
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
