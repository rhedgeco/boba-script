use dashu::integer::IBig;

use crate::{
    engine::{eval::Evaluate, value::ValueKind, EvalError, Value},
    Engine,
};

#[derive(Debug, Clone)]
pub struct Expr<Data> {
    pub kind: Kind<Data>,
    pub data: Data,
}

impl<Data> Kind<Data> {
    pub fn carry(self, data: Data) -> Expr<Data> {
        Expr { kind: self, data }
    }
}

#[derive(Debug, Clone)]
pub enum Kind<Data> {
    // VALUES
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Var(String),
    Tuple(Vec<Expr<Data>>),

    // UNARY OPS
    Pos(Box<Expr<Data>>),
    Neg(Box<Expr<Data>>),
    Not(Box<Expr<Data>>),

    // BINARY OPS
    Add(Box<Expr<Data>>, Box<Expr<Data>>),
    Sub(Box<Expr<Data>>, Box<Expr<Data>>),
    Mul(Box<Expr<Data>>, Box<Expr<Data>>),
    Div(Box<Expr<Data>>, Box<Expr<Data>>),
    Modulo(Box<Expr<Data>>, Box<Expr<Data>>),
    Pow(Box<Expr<Data>>, Box<Expr<Data>>),
    Eq(Box<Expr<Data>>, Box<Expr<Data>>),
    Lt(Box<Expr<Data>>, Box<Expr<Data>>),
    Gt(Box<Expr<Data>>, Box<Expr<Data>>),
    NEq(Box<Expr<Data>>, Box<Expr<Data>>),
    LtEq(Box<Expr<Data>>, Box<Expr<Data>>),
    GtEq(Box<Expr<Data>>, Box<Expr<Data>>),
    And(Box<Expr<Data>>, Box<Expr<Data>>),
    Or(Box<Expr<Data>>, Box<Expr<Data>>),
    Walrus(Box<Expr<Data>>, Box<Expr<Data>>),

    // TERNARY OP
    Ternary {
        cond: Box<Expr<Data>>,
        pass: Box<Expr<Data>>,
        fail: Box<Expr<Data>>,
    },
}

impl<Data: Clone> Evaluate<Data> for Expr<Data> {
    fn eval_with(&self, engine: &mut Engine<Data>) -> Result<Value, EvalError<Data>> {
        match &self.kind {
            // SIMPLE VALUES
            Kind::None => Ok(Value::None),
            Kind::Bool(value) => Ok(Value::Bool(*value)),
            Kind::Int(value) => Ok(Value::Int(value.clone())),
            Kind::Float(value) => Ok(Value::Float(*value)),
            Kind::String(value) => Ok(Value::String(value.clone())),
            Kind::Tuple(exprs) => {
                let mut values = Vec::with_capacity(exprs.len());
                for expr in exprs {
                    values.push(engine.eval(expr)?);
                }
                Ok(Value::Tuple(values))
            }

            // VARIABLES
            Kind::Var(id) => match engine.vars().get(id) {
                Some(value) => Ok(value.clone()),
                None => Err(EvalError::UnknownVariable {
                    data: self.data.clone(),
                    name: id.clone(),
                }),
            },

            // WALRUS
            Kind::Walrus(lhs, rhs) => {
                let value = engine.eval(rhs)?;
                match &lhs.kind {
                    Kind::Var(id) => match engine.vars_mut().set(id, value.clone()) {
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
            Kind::Ternary { cond, pass, fail } => match engine.eval(cond)? {
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
            Kind::Pos(expr) => {
                let inner = engine.eval(expr)?;
                match engine.ops().pos(&inner) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidUnaryOp {
                        ty: inner.kind(),
                        op: "+",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Neg(expr) => {
                let inner = engine.eval(expr)?;
                match engine.ops().neg(&inner) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidUnaryOp {
                        ty: inner.kind(),
                        op: "-",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Not(expr) => {
                let inner = engine.eval(expr)?;
                match engine.ops().not(&inner) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidUnaryOp {
                        ty: inner.kind(),
                        op: "not",
                        data: self.data.clone(),
                    }),
                }
            }

            // BINARY OPS
            Kind::Add(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().add(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "+",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Sub(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().sub(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "-",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Mul(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().mul(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "*",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Div(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().div(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "/",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Modulo(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().modulo(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "%",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Pow(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().pow(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "**",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Eq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().eq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "==",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Lt(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().lt(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "<",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Gt(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().gt(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: ">",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::NEq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().neq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "!=",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::LtEq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().lteq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "<=",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::GtEq(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().gteq(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: ">=",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::And(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().and(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "and",
                        data: self.data.clone(),
                    }),
                }
            }
            Kind::Or(lhs, rhs) => {
                let v1 = engine.eval(lhs)?;
                let v2 = engine.eval(rhs)?;
                match engine.ops().or(&v1, &v2) {
                    Some(value) => Ok(value),
                    None => Err(EvalError::InvalidBinaryOp {
                        ty1: v1.kind(),
                        ty2: v2.kind(),
                        op: "or",
                        data: self.data.clone(),
                    }),
                }
            }
        }
    }
}
