use super::{node::NodeId, Expr, Node};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum StatementKind {
    Invalid,
    Expr(Expr),
    Assign(Expr, Expr),
    If { cond: Expr, body: Vec<Statement> },
    While { cond: Expr, body: Vec<Statement> },
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Statement {
    pub id: NodeId,
    pub kind: StatementKind,
}

impl Node for Statement {
    const NAME: &'static str = "statement";

    fn id(&self) -> NodeId {
        self.id
    }
}

impl Statement {
    pub fn invalid() -> Self {
        Self {
            id: NodeId::new(),
            kind: StatementKind::Invalid,
        }
    }

    pub fn expr(expr: Expr) -> Self {
        Self {
            id: NodeId::new(),
            kind: StatementKind::Expr(expr),
        }
    }

    pub fn assign(lhs: Expr, rhs: Expr) -> Self {
        Self {
            id: NodeId::new(),
            kind: StatementKind::Assign(lhs, rhs),
        }
    }

    pub fn _if(cond: Expr, body: Vec<Statement>) -> Self {
        Self {
            id: NodeId::new(),
            kind: StatementKind::If { cond, body },
        }
    }

    pub fn _while(cond: Expr, body: Vec<Statement>) -> Self {
        Self {
            id: NodeId::new(),
            kind: StatementKind::While { cond, body },
        }
    }
}
