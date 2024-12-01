use crate::{Class, Expr, Func, Module, Node, Pattern};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub enum Definition {
    Static {
        vis: Node<Visibility>,
        pattern: Node<Pattern>,
        expr: Node<Expr>,
    },
    Module {
        vis: Node<Visibility>,
        name: Node<String>,
        module: Node<Module>,
    },
    Class {
        vis: Node<Visibility>,
        name: Node<String>,
        class: Node<Class>,
    },
    Func {
        vis: Node<Visibility>,
        name: Node<String>,
        func: Node<Func>,
    },
}
