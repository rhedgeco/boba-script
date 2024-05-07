use std::{collections::HashMap, ops::Deref};

use derive_more::Display;

use crate::{
    ast::{Expr, Node},
    error::{Color, Label},
    token::Span,
};

use super::{
    types::{Value, ValueType},
    Scope,
};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum UnaryOpType {
    #[display(fmt = "-prefix")]
    Neg,
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

type OpResult = Result<Value, String>;
type UnaryFn = fn(Value) -> OpResult;
type BinaryFn = fn(Value, Value) -> OpResult;

pub struct Engine {
    unary_ops: HashMap<(UnaryOpType, ValueType), UnaryFn>,
    binary_ops: HashMap<(ValueType, BinaryOpType, ValueType), BinaryFn>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval(&self, scope: &Scope, expr: &Node<Expr>) -> Result<Value, Label> {
        match expr.deref() {
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::Float(v) => Ok(Value::Float(*v)),
            Expr::String(v) => Ok(Value::String(v.clone())),
            Expr::Neg(expr) => {
                self.eval_unary(UnaryOpType::Neg, self.eval(scope, expr)?, expr.span())
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
        let val_ty = val.ty();
        match self.unary_ops.get(&(op, val_ty)) {
            Some(op) => op(val),
            None => Err(format!("cannot use unary '{op}' operator with '{val_ty}'")),
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
        let val1_ty = val1.ty();
        let val2_ty = val2.ty();
        match self.binary_ops.get(&(val1_ty, op, val2_ty)) {
            Some(op) => op(val1, val2),
            None => Err(format!(
                "cannot use '{op}' operator with '{val1_ty}' and '{val2_ty}'"
            )),
        }
        .map_err(|message| Label::new(message, Color::Red, span.clone()))
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            unary_ops: HashMap::from([
                // op int
                (
                    (UnaryOpType::Neg, ValueType::Int),
                    (|v| match v {
                        Value::Int(v) => Ok(Value::Int(-v)),
                        _ => unreachable!(),
                    }) as UnaryFn,
                ),
                // op float
                (
                    (UnaryOpType::Neg, ValueType::Float),
                    (|v| match v {
                        Value::Float(v) => Ok(Value::Float(-v)),
                        _ => unreachable!(),
                    }) as UnaryFn,
                ),
            ]),
            binary_ops: HashMap::from([
                // int op int
                (
                    (ValueType::Int, BinaryOpType::Add, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 + v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Sub, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 - v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Mul, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1 * v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Div, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Int(v2)) => Ok(Value::Float(v1 as f64 / v2 as f64)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Pow, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Int(v2)) => {
                            Ok(Value::Float((v1 as f64).powf(v2 as f64)))
                        }
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                // float op float
                (
                    (ValueType::Float, BinaryOpType::Add, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 + v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Sub, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 - v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Mul, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 * v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Div, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1 / v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Pow, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1.powf(v2))),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                // int op float
                (
                    (ValueType::Int, BinaryOpType::Add, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 as f64 + v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Sub, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 as f64 - v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Mul, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 as f64 * v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Div, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Float(v2)) => Ok(Value::Float(v1 as f64 / v2)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Int, BinaryOpType::Pow, ValueType::Float),
                    (|v1, v2| match (v1, v2) {
                        (Value::Int(v1), Value::Float(v2)) => {
                            Ok(Value::Float((v1 as f64).powf(v2)))
                        }
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                // float op int
                (
                    (ValueType::Float, BinaryOpType::Add, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 + v2 as f64)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Sub, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 - v2 as f64)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Mul, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 * v2 as f64)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Div, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1 / v2 as f64)),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                (
                    (ValueType::Float, BinaryOpType::Pow, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::Float(v1), Value::Int(v2)) => Ok(Value::Float(v1.powf(v2 as f64))),
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                // string + string
                (
                    (ValueType::String, BinaryOpType::Add, ValueType::String),
                    (|v1, v2| match (v1, v2) {
                        (Value::String(v1), Value::String(v2)) => {
                            Ok(Value::String(format!("{v1}{v2}")))
                        }
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
                // string * int
                (
                    (ValueType::String, BinaryOpType::Mul, ValueType::Int),
                    (|v1, v2| match (v1, v2) {
                        (Value::String(v1), Value::Int(v2)) => {
                            Ok(Value::String(v1.repeat(v2 as usize)))
                        }
                        _ => unreachable!(),
                    }) as BinaryFn,
                ),
            ]),
        }
    }
}
