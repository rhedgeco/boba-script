use super::{node::NodeId, Node, Statement};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Func {
    pub id: NodeId,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

impl Node for Func {
    const NAME: &'static str = "function";

    fn id(&self) -> NodeId {
        self.id
    }
}

impl Func {
    pub fn new(params: Vec<String>, body: Vec<Statement>) -> Self {
        Self {
            id: NodeId::new(),
            params,
            body,
        }
    }
}
