use std::ops::Deref;

use dashu::integer::IBig;

use crate::{
    engine::{
        value::{FuncPtr, ValueKind},
        EvalError, Value,
    },
    Engine,
};

use super::{func::NodeFunc, node::EvalNode, Node};

pub type ExprNode<Source> = Node<Expr<Source>, Source>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<Source> {
    // VALUES
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Var(String),
    Tuple(Vec<ExprNode<Source>>),
    Func(NodeFunc<Source>),

    // UNARY OPS
    Pos(Box<ExprNode<Source>>),
    Neg(Box<ExprNode<Source>>),
    Not(Box<ExprNode<Source>>),

    // BINARY OPS
    Add(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Sub(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Mul(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Div(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Modulo(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Pow(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Eq(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Lt(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Gt(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    NEq(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    LtEq(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    GtEq(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    And(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Or(Box<ExprNode<Source>>, Box<ExprNode<Source>>),
    Walrus(Box<ExprNode<Source>>, Box<ExprNode<Source>>),

    // TERNARY OP
    Ternary {
        cond: Box<ExprNode<Source>>,
        pass: Box<ExprNode<Source>>,
        fail: Box<ExprNode<Source>>,
    },

    // FUNCTION CALL
    Call {
        name: String,
        params: Vec<ExprNode<Source>>,
    },
}

impl<Source: Clone> EvalNode<Source> for Expr<Source> {
    fn eval_node(
        node: &Node<Self, Source>,
        engine: &mut Engine<Source>,
    ) -> Result<Value<Source>, EvalError<Source>> {
        match &node.item {
            // SIMPLE VALUES
            Expr::None => Ok(Value::None),
            Expr::Bool(value) => Ok(Value::Bool(*value)),
            Expr::Int(value) => Ok(Value::Int(value.clone())),
            Expr::Float(value) => Ok(Value::Float(*value)),
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Func(func) => Ok(Value::Func(FuncPtr::custom(func.deref().clone()))),
            Expr::Tuple(exprs) => {
                let mut values = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    values.push(engine.eval(expr)?);
                }
                Ok(Value::Tuple(values.into_iter().collect()))
            }

            // VARIABLES
            Expr::Var(id) => match engine.vars().get(id) {
                Some(value) => Ok(value.clone()),
                None => Err(EvalError::UnknownVariable {
                    source: node.source.clone(),
                    name: id.clone(),
                }),
            },

            // FUNCTION CALL
            Expr::Call { name, params } => match engine.vars().get(name.deref()) {
                Some(Value::Func(func)) => {
                    let func = func.clone();
                    let mut values = Vec::new();
                    for expr in params.iter() {
                        values.push(engine.eval(expr)?)
                    }
                    func.call(&node.source, values, engine)
                }
                Some(value) => Err(EvalError::NotAFunction {
                    name: name.clone(),
                    found: value.kind(),
                    source: node.source.clone(),
                }),
                None => Err(EvalError::UnknownFunction {
                    source: node.source.clone(),
                    name: name.to_string(),
                }),
            },

            // WALRUS
            Expr::Walrus(lhs, rhs) => {
                let value = engine.eval(rhs)?;
                match &lhs.item {
                    Expr::Var(id) => match engine.vars_mut().set(id, value.clone()) {
                        Ok(_) => Ok(value),
                        Err(_) => Err(EvalError::UnknownVariable {
                            source: lhs.source.clone(),
                            name: id.clone(),
                        }),
                    },
                    _ => Err(EvalError::InvalidAssign {
                        source: lhs.source.clone(),
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
                    source: cond.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
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
                        source: node.source.clone(),
                    }),
                }
            }
        }
    }
}
