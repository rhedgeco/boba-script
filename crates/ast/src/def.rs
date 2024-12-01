use crate::{statement::Assignment, Class, Func, Module, Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub enum Definition {
    Static {
        vis: Node<Visibility>,
        assign: Node<Assignment>,
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
