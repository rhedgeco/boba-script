use std::ops::Deref;

use derive_more::Display;

use crate::{
    error::{Color, Label},
    lexer::token::Span,
    parser::{Expr, Node},
};

use super::{types::Value, Scope};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum UnaryOpType {
    #[display(fmt = "-")]
    Neg,
    #[display(fmt = "!")]
    Not,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum BinaryOpType {
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

#[derive(Debug, Default)]
pub struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval(&self, scope: &Scope, expr: &Node<Expr>) -> Result<Value, Label> {
        match expr.deref() {
            Expr::Bool(v) => Ok(Value::Bool(*v)),
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::Float(v) => Ok(Value::Float(*v)),
            Expr::String(v) => Ok(Value::String(v.clone())),
            Expr::Neg(inner) => {
                self.eval_unary(UnaryOpType::Neg, self.eval(scope, inner)?, expr.span())
            }
            Expr::Not(inner) => {
                self.eval_unary(UnaryOpType::Not, self.eval(scope, inner)?, expr.span())
            }
            Expr::Add(lhs, rhs) => self.eval_binary(
                self.eval(scope, lhs)?,
                BinaryOpType::Add,
                self.eval(scope, rhs)?,
                expr.span(),
            ),
            Expr::Sub(lhs, rhs) => self.eval_binary(
                self.eval(scope, lhs)?,
                BinaryOpType::Sub,
                self.eval(scope, rhs)?,
                expr.span(),
            ),
            Expr::Mul(lhs, rhs) => self.eval_binary(
                self.eval(scope, lhs)?,
                BinaryOpType::Mul,
                self.eval(scope, rhs)?,
                expr.span(),
            ),
            Expr::Div(lhs, rhs) => self.eval_binary(
                self.eval(scope, lhs)?,
                BinaryOpType::Div,
                self.eval(scope, rhs)?,
                expr.span(),
            ),
            Expr::Pow(lhs, rhs) => self.eval_binary(
                self.eval(scope, lhs)?,
                BinaryOpType::Pow,
                self.eval(scope, rhs)?,
                expr.span(),
            ),
            Expr::Var(ident) => match scope.get_var(ident) {
                Some(value) => Ok(value.clone()),
                None => Err(Label::new(
                    format!("Unknown variable '{ident}'"),
                    Color::Red,
                    expr.span().clone(),
                )),
            },
        }
    }

    fn eval_unary(&self, op: UnaryOpType, val: Value, span: &Span) -> Result<Value, Label> {
        let val_ty = val.type_name();
        match (val, op) {
            (Value::Bool(v), UnaryOpType::Not) => Ok(Value::Bool(!v)),
            (Value::Int(v), UnaryOpType::Neg) => Ok(Value::Int(-v)),
            (Value::Float(v), UnaryOpType::Neg) => Ok(Value::Float(-v)),
            _ => Err(format!("cannot use unary '{op}' prefix with '{val_ty}'",)),
        }
        .map_err(|message| Label::new(message, Color::Red, span.clone()))
    }

    fn eval_binary(
        &self,
        val1: Value,
        op: BinaryOpType,
        val2: Value,
        span: &Span,
    ) -> Result<Value, Label> {
        let val1_ty = val1.type_name();
        let val2_ty = val2.type_name();

        match (val1, op, val2) {
            // ---------------
            // --- INT OPS ---
            // int add
            (Value::Int(v1), BinaryOpType::Add, Value::Int(v2)) => Ok(Value::Int(v1 + v2)),
            (Value::Int(v1), BinaryOpType::Add, Value::Float(v2)) => {
                Ok(Value::Float(v1 as f64 + v2))
            }
            // int sub
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
            (Value::Float(v1), BinaryOpType::Add, Value::Int(v2)) => {
                Ok(Value::Float(v1 + v2 as f64))
            }
            (Value::Float(v1), BinaryOpType::Add, Value::Float(v2)) => Ok(Value::Float(v1 + v2)),
            // float sub
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
            _ => Err(format!(
                "'{val1_ty}' does not have a valid '{op}' operator for '{val2_ty}'",
            )),
        }
        .map_err(|message| Label::new(message, Color::Red, span.clone()))
    }
}
