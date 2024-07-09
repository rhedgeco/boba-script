use dashu::integer::IBig;

use crate::{
    engine::{value::ValueKind, EvalError, Value},
    Engine,
};

use super::{node::EvalNode, Node};

pub type ExprNode<Data> = Node<Expr<Data>, Data>;

#[derive(Debug, Clone)]
pub enum Expr<Data> {
    // VALUES
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Var(String),
    Tuple(Vec<ExprNode<Data>>),

    // UNARY OPS
    Pos(Box<ExprNode<Data>>),
    Neg(Box<ExprNode<Data>>),
    Not(Box<ExprNode<Data>>),

    // BINARY OPS
    Add(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Sub(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Mul(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Div(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Modulo(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Pow(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Eq(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Lt(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Gt(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    NEq(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    LtEq(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    GtEq(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    And(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Or(Box<ExprNode<Data>>, Box<ExprNode<Data>>),
    Walrus(Box<ExprNode<Data>>, Box<ExprNode<Data>>),

    // TERNARY OP
    Ternary {
        cond: Box<ExprNode<Data>>,
        pass: Box<ExprNode<Data>>,
        fail: Box<ExprNode<Data>>,
    },
}

impl<Data: Clone> EvalNode<Data> for Expr<Data> {
    fn eval_node(
        node: &Node<Self, Data>,
        engine: &mut Engine<Data>,
    ) -> Result<Value, EvalError<Data>> {
        match &node.item {
            // SIMPLE VALUES
            Expr::None => Ok(Value::None),
            Expr::Bool(value) => Ok(Value::Bool(*value)),
            Expr::Int(value) => Ok(Value::Int(value.clone())),
            Expr::Float(value) => Ok(Value::Float(*value)),
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Tuple(exprs) => {
                let mut values = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    values.push(engine.eval(expr)?);
                }
                Ok(Value::Tuple(values))
            }

            // VARIABLES
            Expr::Var(id) => match engine.vars().get(id) {
                Some(value) => Ok(value.clone()),
                None => Err(EvalError::UnknownVariable {
                    data: node.data.clone(),
                    name: id.clone(),
                }),
            },

            // WALRUS
            Expr::Walrus(lhs, rhs) => {
                let value = engine.eval(rhs)?;
                match &lhs.item {
                    Expr::Var(id) => match engine.vars_mut().set(id, value.clone()) {
                        Ok(_) => Ok(value),
                        Err(_) => Err(EvalError::UnknownVariable {
                            data: lhs.data.clone(),
                            name: id.clone(),
                        }),
                    },
                    _ => Err(EvalError::AssignError {
                        data: lhs.data.clone(),
                    }),
                }
            }

            // TERNARY
            Expr::Ternary { cond, pass, fail } => match engine.eval(cond)? {
                Value::Bool(bool) => match bool {
                    true => engine.eval(pass),
                    false => engine.eval(fail),
                },
                value => Err(EvalError::UnexpectedType {
                    expect: ValueKind::Bool,
                    found: value.kind(),
                    data: cond.data.clone(),
                }),
            },

            // UNARY OPS
            Expr::Pos(expr) => {
                let inner = engine.eval(expr)?;
                match engine.ops().pos(&inner) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidUnaryOp {
                        ty: inner.kind(),
                        op: "+",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Neg(expr) => {
                let inner = engine.eval(expr)?;
                match engine.ops().neg(&inner) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidUnaryOp {
                        ty: inner.kind(),
                        op: "-",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Not(expr) => {
                let inner = engine.eval(expr)?;
                match engine.ops().not(&inner) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidUnaryOp {
                        ty: inner.kind(),
                        op: "not",
                        data: node.data.clone(),
                    }),
                }
            }

            // BINARY OPS
            Expr::Add(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().add(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "+",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Sub(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().sub(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "-",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Mul(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().mul(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "*",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Div(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().div(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "/",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Modulo(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().modulo(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "%",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Pow(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().pow(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "**",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Eq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().eq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "==",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Lt(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().lt(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "<",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Gt(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().gt(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: ">",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::NEq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().neq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "!=",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::LtEq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().lteq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "<=",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::GtEq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().gteq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: ">=",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::And(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().and(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "and",
                        data: node.data.clone(),
                    }),
                }
            }
            Expr::Or(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().or(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "or",
                        data: node.data.clone(),
                    }),
                }
            }
        }
    }
}
