use crate::{Class, Expr, Func, Module, Node, Visibility};

#[derive(Debug, Clone)]
pub struct Definition {
    pub vis: Node<Visibility>,
    pub name: Node<String>,
    pub kind: DefKind,
}

#[derive(Debug, Clone)]
pub enum DefKind {
    Global(Node<Expr>),
    Module(Node<Module>),
    Class(Node<Class>),
    Func(Node<Func>),
}
