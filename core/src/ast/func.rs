use std::fmt::Display;

use crate::{
    engine::{EvalError, Value},
    Engine,
};

use super::{node::EvalNode, Node, StatementNode};

#[derive(Debug, Clone)]
pub struct Func<Source> {
    pub name: Node<String, Source>,
    pub params: Vec<Node<String, Source>>,
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

        write!(f, "fn {}{params}", self.name)
    }
}

impl<Source: Clone> EvalNode<Source> for Func<Source> {
    fn eval_node(
        _node: &Node<Self, Source>,
        _: &mut Engine<Source>,
    ) -> Result<Value<Source>, EvalError<Source>> {
        todo!()
    }
}
