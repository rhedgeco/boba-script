use std::ops::Deref;

use derive_more::Display;

use crate::{
    ast::{Expr, Ident},
    parser::Node,
    token::Span,
};

use super::{
    scope::{EngineScope, ScopeGroup},
    types::Value,
    RunError, Scope,
};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryOpType {
    #[display(fmt = "-")]
    Neg,
    #[display(fmt = "!")]
    Not,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryOpType {
    #[display(fmt = "+")]
    Add,
    #[display(fmt = "-")]
    Sub,
    #[display(fmt = "*")]
    Mul,
    #[display(fmt = "/")]
    Div,
    #[display(fmt = "**")]
    Pow,
}

pub enum CallError {
    Runtime(RunError),
    NotFound,
}

#[derive(Debug, Default)]
pub struct Engine {
    scope: Scope,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn insert_var(&mut self, ident: Ident, value: Value) {
        self.scope.init_var(ident, value);
    }

    pub fn eval(&self, expr: &Node<Expr>) -> Result<Value, RunError> {
        self.eval_with_scope(expr, &self.scope)
    }

    pub fn eval_with_scope(
        &self,
        expr: &Node<Expr>,
        scope: &impl EngineScope,
    ) -> Result<Value, RunError> {
        match expr.deref() {
            Expr::Bool(v) => Ok(Value::Bool(*v)),
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::Float(v) => Ok(Value::Float(*v)),
            Expr::String(v) => Ok(Value::String(v.clone())),
            Expr::Neg(inner) => self.eval_unary(
                UnaryOpType::Neg,
                self.eval_with_scope(inner, scope)?,
                expr.span(),
            ),
            Expr::Not(inner) => self.eval_unary(
                UnaryOpType::Not,
                self.eval_with_scope(inner, scope)?,
                expr.span(),
            ),
            Expr::Add(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Add,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Sub(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Sub,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Mul(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Mul,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Div(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Div,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Pow(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Pow,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Var(ident) => match ScopeGroup::new(scope, &self.scope).get_var(ident) {
                Some(value) => Ok(value.clone()),
                None => Err(RunError::UnknownVariable {
                    ident: ident.clone(),
                    span: expr.span().clone(),
                }),
            },
            _ => todo!(),
        }
    }

    fn eval_unary(&self, op: UnaryOpType, val: Value, span: &Span) -> Result<Value, RunError> {
        let vtype = val.type_name();
        match (val, op) {
            (Value::Bool(v), UnaryOpType::Not) => Ok(Value::Bool(!v)),
            (Value::Int(v), UnaryOpType::Neg) => Ok(Value::Int(-v)),
            (Value::Float(v), UnaryOpType::Neg) => Ok(Value::Float(-v)),
            _ => Err(RunError::InvalidUnary {
                op,
                vtype,
                span: span.clone(),
            }),
        }
    }

    fn eval_binary(
        &self,
        val1: Value,
        op: BinaryOpType,
        val2: Value,
        span: &Span,
    ) -> Result<Value, RunError> {
        let vtype1 = val1.type_name();
        let vtype2 = val2.type_name();

        match (val1, op, val2) {
            // ---------------
            // --- INT OPS ---
            // int add
            (Value::Int(v1), BinaryOpType::Add, Value::Bool(v2)) => Ok(Value::Int(v1 + v2 as i64)),
            (Value::Int(v1), BinaryOpType::Add, Value::Int(v2)) => Ok(Value::Int(v1 + v2)),
            (Value::Int(v1), BinaryOpType::Add, Value::Float(v2)) => {
                Ok(Value::Float(v1 as f64 + v2))
            }
            // int sub
            (Value::Int(v1), BinaryOpType::Sub, Value::Bool(v2)) => Ok(Value::Int(v1 - v2 as i64)),
            (Value::Int(v1), BinaryOpType::Sub, Value::Int(v2)) => Ok(Value::Int(v1 - v2)),
            (Value::Int(v1), BinaryOpType::Sub, Value::Float(v2)) => {
                Ok(Value::Float(v1 as f64 - v2))
            }
            // int mul
            (Value::Int(v1), BinaryOpType::Mul, Value::Bool(v2)) => Ok(Value::Int(v1 * v2 as i64)),
            (Value::Int(v1), BinaryOpType::Mul, Value::Int(v2)) => Ok(Value::Int(v1 * v2)),
            (Value::Int(v1), BinaryOpType::Mul, Value::Float(v2)) => {
                Ok(Value::Float(v1 as f64 * v2))
            }
            // int div
            (Value::Int(v1), BinaryOpType::Div, Value::Int(v2)) => {
                Ok(Value::Float(v1 as f64 / v2 as f64))
            }
            (Value::Int(v1), BinaryOpType::Div, Value::Float(v2)) => {
                Ok(Value::Float(v1 as f64 / v2))
            }
            // int pow
            (Value::Int(v1), BinaryOpType::Pow, Value::Int(v2)) => {
                Ok(Value::Float((v1 as f64).powf(v2 as f64)))
            }
            (Value::Int(v1), BinaryOpType::Pow, Value::Float(v2)) => {
                Ok(Value::Float((v1 as f64).powf(v2)))
            }

            // -----------------
            // --- FLOAT OPS ---
            // float add
            (Value::Float(v1), BinaryOpType::Add, Value::Bool(v2)) => {
                Ok(Value::Float(v1 + v2 as i64 as f64))
            }
            (Value::Float(v1), BinaryOpType::Add, Value::Int(v2)) => {
                Ok(Value::Float(v1 + v2 as f64))
            }
            (Value::Float(v1), BinaryOpType::Add, Value::Float(v2)) => Ok(Value::Float(v1 + v2)),
            // float sub
            (Value::Float(v1), BinaryOpType::Sub, Value::Bool(v2)) => {
                Ok(Value::Float(v1 - v2 as i64 as f64))
            }
            (Value::Float(v1), BinaryOpType::Sub, Value::Int(v2)) => {
                Ok(Value::Float(v1 - v2 as f64))
            }
            (Value::Float(v1), BinaryOpType::Sub, Value::Float(v2)) => Ok(Value::Float(v1 - v2)),
            // float mul
            (Value::Float(v1), BinaryOpType::Mul, Value::Bool(v2)) => {
                Ok(Value::Float(v1 * v2 as i64 as f64))
            }
            (Value::Float(v1), BinaryOpType::Mul, Value::Int(v2)) => {
                Ok(Value::Float(v1 * v2 as f64))
            }
            (Value::Float(v1), BinaryOpType::Mul, Value::Float(v2)) => Ok(Value::Float(v1 * v2)),
            // float div
            (Value::Float(v1), BinaryOpType::Div, Value::Int(v2)) => {
                Ok(Value::Float(v1 / v2 as f64))
            }
            (Value::Float(v1), BinaryOpType::Div, Value::Float(v2)) => Ok(Value::Float(v1 / v2)),
            // float pow
            (Value::Float(v1), BinaryOpType::Pow, Value::Int(v2)) => {
                Ok(Value::Float(v1.powf(v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::Pow, Value::Float(v2)) => {
                Ok(Value::Float(v1.powf(v2)))
            }

            // ------------------
            // --- STRING OPS ---
            // string add
            (Value::String(v1), BinaryOpType::Add, Value::Bool(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            (Value::String(v1), BinaryOpType::Add, Value::Int(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            (Value::String(v1), BinaryOpType::Add, Value::Float(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            (Value::String(v1), BinaryOpType::Add, Value::String(v2)) => {
                Ok(Value::String(format!("{v1}{v2}")))
            }
            // string mul
            (Value::String(v1), BinaryOpType::Mul, Value::Bool(v2)) => {
                Ok(Value::String(v1.repeat(v2 as usize)))
            }
            (Value::String(v1), BinaryOpType::Mul, Value::Int(v2)) => {
                Ok(Value::String(v1.repeat(v2 as usize)))
            }

            // --------------------
            // --- FAILURE CASE ---
            _ => Err(RunError::InvalidBinary {
                op,
                vtype1,
                vtype2,
                span: span.clone(),
            }),
        }
    }
}
