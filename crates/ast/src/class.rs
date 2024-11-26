use crate::{
    def::Visibility,
    field::InitField,
    path::{ConcreteType, Union},
    Definition, Node, Pattern,
};

pub struct Class {
    pub fields: Vec<Node<ClassField>>,
    pub defs: Vec<Node<Definition>>,
}

pub struct ClassInit {
    pub name: Node<String>,
    pub fields: Vec<Node<InitField>>,
}

pub struct ClassPattern {
    pub name: Node<ConcreteType>,
    pub fields: Vec<Node<Pattern>>,
}

pub struct ClassField {
    pub vis: Node<Visibility>,
    pub name: Node<String>,
    pub union: Node<Union>,
}
