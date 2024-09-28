use dashu_int::IBig;

use crate::Func;

use super::{node::NodeId, Node};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ExprKind {
    // INVALID
    Invalid,

    // VALUES
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Fn(Func),
    Var(String),

    // UNARY OPS
    Pos(Box<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),

    // BINARY OPS
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    NEq(Box<Expr>, Box<Expr>),
    LtEq(Box<Expr>, Box<Expr>),
    GtEq(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Walrus(Box<Expr>, Box<Expr>),

    // TERNARY
    Ternary {
        cond: Box<Expr>,
        pass: Box<Expr>,
        fail: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
}

impl Node for Expr {
    const NAME: &'static str = "expression";

    fn id(&self) -> NodeId {
        self.id
    }
}

impl Expr {
    pub fn invalid() -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Invalid,
        }
    }

    pub fn none() -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::None,
        }
    }

    pub fn bool(bool: bool) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Bool(bool),
        }
    }

    pub fn int(int: IBig) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Int(int),
        }
    }

    pub fn float(float: f64) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Float(float),
        }
    }

    pub fn str(string: String) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::String(string),
        }
    }

    pub fn _fn(func: Func) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Fn(func),
        }
    }

    pub fn var(string: String) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Var(string),
        }
    }

    pub fn pos(self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Pos(Box::new(self)),
        }
    }

    pub fn neg(self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Neg(Box::new(self)),
        }
    }

    pub fn not(self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Not(Box::new(self)),
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Add(Box::new(self), Box::new(other)),
        }
    }

    pub fn sub(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Sub(Box::new(self), Box::new(other)),
        }
    }

    pub fn mul(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Mul(Box::new(self), Box::new(other)),
        }
    }

    pub fn div(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Div(Box::new(self), Box::new(other)),
        }
    }

    pub fn _mod(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Mod(Box::new(self), Box::new(other)),
        }
    }

    pub fn pow(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Pow(Box::new(self), Box::new(other)),
        }
    }

    pub fn eq(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Eq(Box::new(self), Box::new(other)),
        }
    }

    pub fn lt(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Lt(Box::new(self), Box::new(other)),
        }
    }

    pub fn gt(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Gt(Box::new(self), Box::new(other)),
        }
    }

    pub fn neq(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::NEq(Box::new(self), Box::new(other)),
        }
    }

    pub fn lteq(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::LtEq(Box::new(self), Box::new(other)),
        }
    }

    pub fn gteq(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::GtEq(Box::new(self), Box::new(other)),
        }
    }

    pub fn and(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::And(Box::new(self), Box::new(other)),
        }
    }

    pub fn or(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Or(Box::new(self), Box::new(other)),
        }
    }

    pub fn walrus(self, other: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Walrus(Box::new(self), Box::new(other)),
        }
    }

    pub fn ternary(self, pass: Self, fail: Self) -> Self {
        Self {
            id: NodeId::new(),
            kind: ExprKind::Ternary {
                cond: Box::new(self),
                pass: Box::new(pass),
                fail: Box::new(fail),
            },
        }
    }
}
