use crate::{
    def::Visibility, field::InitField, union::ConcreteType, Definition, Node, Pattern, Union,
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
    pub ty: Node<Union>,
}
