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
    #[display(fmt = "%")]
    Mod,
    #[display(fmt = "**")]
    Pow,

    #[display(fmt = "==")]
    Eq,
    #[display(fmt = "<")]
    Lt,
    #[display(fmt = ">")]
    Gt,
    #[display(fmt = "!=")]
    NEq,
    #[display(fmt = "<=")]
    LtEq,
    #[display(fmt = ">=")]
    GtEq,
    #[display(fmt = "and")]
    And,
    #[display(fmt = "or")]
    Or,
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
            Expr::Mod(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Mod,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Eq(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Eq,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Lt(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Lt,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Gt(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Gt,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::NEq(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::NEq,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::LtEq(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::LtEq,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::GtEq(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::GtEq,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::And(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::And,
                self.eval_with_scope(rhs, scope)?,
                expr.span(),
            ),
            Expr::Or(lhs, rhs) => self.eval_binary(
                self.eval_with_scope(lhs, scope)?,
                BinaryOpType::Or,
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
            Expr::Ternary(lhs, cond, rhs) => {
                // evaluate condition
                let cond = match self.eval(&cond)? {
                    Value::Bool(cond) => cond,
                    value => {
                        return Err(RunError::UnexpectedType {
                            expected: "'bool'".into(),
                            found: format!("'{}'", value.type_name()),
                            span: cond.span().clone(),
                        })
                    }
                };

                // then evaluate the correct expression
                match cond {
                    true => self.eval(&lhs),
                    false => self.eval(&rhs),
                }
            }
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
            // int mod
            (Value::Int(v1), BinaryOpType::Mod, Value::Int(v2)) => {
                Ok(Value::Int(v1.rem_euclid(v2)))
            }
            (Value::Int(v1), BinaryOpType::Mod, Value::Float(v2)) => {
                Ok(Value::Float((v1 as f64).rem_euclid(v2)))
            }
            // int pow
            (Value::Int(v1), BinaryOpType::Pow, Value::Int(v2)) => {
                Ok(Value::Float((v1 as f64).powf(v2 as f64)))
            }
            (Value::Int(v1), BinaryOpType::Pow, Value::Float(v2)) => {
                Ok(Value::Float((v1 as f64).powf(v2)))
            }
            // int equality
            (Value::Int(v1), BinaryOpType::Eq, Value::Int(v2)) => Ok(Value::Bool(v1 == v2)),
            (Value::Int(v1), BinaryOpType::Eq, Value::Float(v2)) => {
                Ok(Value::Bool((v1 as f64) == v2))
            }
            // int less than
            (Value::Int(v1), BinaryOpType::Lt, Value::Int(v2)) => Ok(Value::Bool(v1 < v2)),
            (Value::Int(v1), BinaryOpType::Lt, Value::Float(v2)) => {
                Ok(Value::Bool((v1 as f64) < v2))
            }
            // int greater than
            (Value::Int(v1), BinaryOpType::Gt, Value::Int(v2)) => Ok(Value::Bool(v1 > v2)),
            (Value::Int(v1), BinaryOpType::Gt, Value::Float(v2)) => {
                Ok(Value::Bool((v1 as f64) > v2))
            }
            // int not equal
            (Value::Int(v1), BinaryOpType::NEq, Value::Int(v2)) => Ok(Value::Bool(v1 != v2)),
            (Value::Int(v1), BinaryOpType::NEq, Value::Float(v2)) => {
                Ok(Value::Bool((v1 as f64) != v2))
            }
            // int less than equal
            (Value::Int(v1), BinaryOpType::LtEq, Value::Int(v2)) => Ok(Value::Bool(v1 <= v2)),
            (Value::Int(v1), BinaryOpType::LtEq, Value::Float(v2)) => {
                Ok(Value::Bool((v1 as f64) <= v2))
            }
            // int greater than equal
            (Value::Int(v1), BinaryOpType::GtEq, Value::Int(v2)) => Ok(Value::Bool(v1 >= v2)),
            (Value::Int(v1), BinaryOpType::GtEq, Value::Float(v2)) => {
                Ok(Value::Bool((v1 as f64) >= v2))
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
            // float mod
            (Value::Float(v1), BinaryOpType::Mod, Value::Int(v2)) => {
                Ok(Value::Float(v1.rem_euclid(v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::Mod, Value::Float(v2)) => {
                Ok(Value::Float(v1.rem_euclid(v2)))
            }
            // float pow
            (Value::Float(v1), BinaryOpType::Pow, Value::Int(v2)) => {
                Ok(Value::Float(v1.powf(v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::Pow, Value::Float(v2)) => {
                Ok(Value::Float(v1.powf(v2)))
            }
            // float equality
            (Value::Float(v1), BinaryOpType::Eq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 == (v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::Eq, Value::Float(v2)) => Ok(Value::Bool(v1 == v2)),
            // float less than
            (Value::Float(v1), BinaryOpType::Lt, Value::Int(v2)) => {
                Ok(Value::Bool(v1 < (v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::Lt, Value::Float(v2)) => Ok(Value::Bool(v1 < v2)),
            // float greater than
            (Value::Float(v1), BinaryOpType::Gt, Value::Int(v2)) => {
                Ok(Value::Bool(v1 > (v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::Gt, Value::Float(v2)) => Ok(Value::Bool(v1 > v2)),
            // float not equal
            (Value::Float(v1), BinaryOpType::NEq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 != (v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::NEq, Value::Float(v2)) => Ok(Value::Bool(v1 != v2)),
            // float less than equal
            (Value::Float(v1), BinaryOpType::LtEq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 <= (v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::LtEq, Value::Float(v2)) => Ok(Value::Bool(v1 <= v2)),
            // float greater than equal
            (Value::Float(v1), BinaryOpType::GtEq, Value::Int(v2)) => {
                Ok(Value::Bool(v1 >= (v2 as f64)))
            }
            (Value::Float(v1), BinaryOpType::GtEq, Value::Float(v2)) => Ok(Value::Bool(v1 >= v2)),

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
            // string equality
            (Value::String(v1), BinaryOpType::Eq, Value::String(v2)) => Ok(Value::Bool(v1 == v2)),
            // string less than
            (Value::String(v1), BinaryOpType::Lt, Value::String(v2)) => Ok(Value::Bool(v1 < v2)),
            // string greater than
            (Value::String(v1), BinaryOpType::Gt, Value::String(v2)) => Ok(Value::Bool(v1 > v2)),
            // string not equal
            (Value::String(v1), BinaryOpType::NEq, Value::String(v2)) => Ok(Value::Bool(v1 != v2)),
            // string less than equal
            (Value::String(v1), BinaryOpType::LtEq, Value::String(v2)) => Ok(Value::Bool(v1 <= v2)),
            // string greater than equal
            (Value::String(v1), BinaryOpType::GtEq, Value::String(v2)) => Ok(Value::Bool(v1 >= v2)),

            // -------------------
            // --- BOOLEAN OPS ---
            // boolean equality
            (Value::Bool(v1), BinaryOpType::Eq, Value::Bool(v2)) => Ok(Value::Bool(v1 == v2)),
            // boolean less than
            (Value::Bool(v1), BinaryOpType::Lt, Value::Bool(v2)) => Ok(Value::Bool(v1 < v2)),
            // boolean greater than
            (Value::Bool(v1), BinaryOpType::Gt, Value::Bool(v2)) => Ok(Value::Bool(v1 > v2)),
            // boolean not equal
            (Value::Bool(v1), BinaryOpType::NEq, Value::Bool(v2)) => Ok(Value::Bool(v1 != v2)),
            // boolean less than equal
            (Value::Bool(v1), BinaryOpType::LtEq, Value::Bool(v2)) => Ok(Value::Bool(v1 <= v2)),
            // boolean greater than equal
            (Value::Bool(v1), BinaryOpType::GtEq, Value::Bool(v2)) => Ok(Value::Bool(v1 >= v2)),
            // boolean and
            (Value::Bool(v1), BinaryOpType::And, Value::Bool(v2)) => Ok(Value::Bool(v1 && v2)),
            // boolean or
            (Value::Bool(v1), BinaryOpType::Or, Value::Bool(v2)) => Ok(Value::Bool(v1 || v2)),

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
