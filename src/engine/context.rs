use std::{any::Any, collections::HashMap, ops::Deref};

use ariadne::{Label, Report, ReportKind, Source};

use crate::{
    ast::{Color, Expr, Node},
    token::{Ident, Span},
};

use super::types::{ExprOp, ExprOpType, Value};

pub struct OpError {
    pub message: String,
}

impl OpError {
    pub fn report(
        &self,
        span: Span,
        src_id: impl AsRef<str>,
        source: Source<impl AsRef<str>>,
    ) -> Report<(&str, Span)> {
        let src_id = src_id.as_ref();
        Report::build(ReportKind::Error, src_id, span.start)
            .with_message("Operation Error")
            .with_label(
                Label::new((src_id, span))
                    .with_color(Color::Red)
                    .with_message(self.message),
            )
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct Vars {
    vars: Vec<(Ident, Value)>,
}

impl Vars {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_var(&mut self, ident: Ident, value: Value) {
        self.vars.push((ident, value));
    }

    pub fn set_var(&mut self, ident: &Ident, value: Value) -> Option<Value> {
        match self.vars.iter_mut().rev().find(|(id, _)| id == ident) {
            None => Some(value),
            Some((_, old_value)) => {
                *old_value = value;
                None
            }
        }
    }

    pub fn get_var(&mut self, ident: &Ident) -> Option<&Value> {
        self.vars
            .iter()
            .rev()
            .find(|(id, _)| id == ident)
            .map(|(_, value)| value)
    }

    pub fn pop_vars(&mut self, count: usize) {
        let new_len = self.vars.len().saturating_sub(count);
        self.vars.truncate(new_len);
    }
}

pub struct Context {
    ops: HashMap<ExprOpType, Box<dyn Any>>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval_expr(&self, expr: &Node<Expr>, scope: &mut Vars) -> Result<Value, OpError> {
        fn eval_op<T1, T2>(ctx: &Context, expr: ExprOp<T1, T2>) -> Result<Value, OpError>
        where
            Value: From<T1>,
            Value: From<T2>,
        {
            let expr_ty = expr.ty();
            match ctx.ops.get(&expr_ty) {
                Some(op) => (op
                    .downcast_ref::<fn(ExprOp<T1, T2>) -> Result<Value, OpError>>()
                    .unwrap())(expr),
                None => Err(OpError {
                    message: format!("no op function for '{expr_ty}'"),
                }),
            }
        }

        match expr.deref() {
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::Float(v) => Ok(Value::Float(*v)),
            Expr::Neg(expr) => eval_op(self, ExprOp::<_, ()>::Neg(self.eval_expr(expr, scope)?)),
            Expr::Add(lhs, rhs) => eval_op(
                self,
                ExprOp::Add(self.eval_expr(lhs, scope)?, self.eval_expr(rhs, scope)?),
            ),
            Expr::Sub(lhs, rhs) => eval_op(
                self,
                ExprOp::Add(self.eval_expr(lhs, scope)?, self.eval_expr(rhs, scope)?),
            ),
            Expr::Mul(lhs, rhs) => eval_op(
                self,
                ExprOp::Add(self.eval_expr(lhs, scope)?, self.eval_expr(rhs, scope)?),
            ),
            Expr::Div(lhs, rhs) => eval_op(
                self,
                ExprOp::Add(self.eval_expr(lhs, scope)?, self.eval_expr(rhs, scope)?),
            ),
            Expr::Pow(lhs, rhs) => eval_op(
                self,
                ExprOp::Add(self.eval_expr(lhs, scope)?, self.eval_expr(rhs, scope)?),
            ),
            Expr::Var(ident) => scope.get_var(ident).map(|v| *v).ok_or_else(|| OpError {
                message: format!(""),
            }),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            ops: todo!(),
            // ops: HashMap::from([
            //     // int op int
            //     (
            //         ExprOpType::Add(ValueType::Int, ValueType::Int),
            //         (|expr| match expr {
            //             ExprOp::Add(v1, v2) => Ok(Value::Int(v1 + v2)),
            //             _ => unreachable!(),
            //         }) as fn(ExprOp<i64, i64>) -> Result<Value, OpError>,
            //     ),
            // ]),
        }
    }
}
