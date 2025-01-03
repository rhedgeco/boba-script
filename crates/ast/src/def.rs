use crate::{Class, Expr, Func, Module, Node, Visibility};

#[derive(Debug, Clone)]
pub struct Definition<Type> {
    pub vis: Node<Visibility>,
    pub name: Node<String>,
    pub kind: DefKind<Type>,
}

#[derive(Debug, Clone)]
pub enum DefKind<Type> {
    Global(Node<Expr<Type>>),
    Module(Node<Module<Type>>),
    Class(Node<Class<Type>>),
    Func(Node<Func<Type>>),
}
