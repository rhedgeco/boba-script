use std::fmt::Display;

use super::{Node, StatementNode};

pub type NodeFunc<Source> = Node<Func<Source>, Source>;

#[derive(Debug, Clone, PartialEq)]
pub struct Func<Source> {
    pub params: Vec<String>,
    pub body: Vec<StatementNode<Source>>,
}

impl<Source> Display for Func<Source> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| format!("{p}"))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "fn({params})")
    }
}
