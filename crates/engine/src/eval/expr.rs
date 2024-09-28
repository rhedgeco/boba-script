use std::usize;

use boba_script_ast::{expr::ExprKind, int::Sign, Expr, Node};
use num_traits::ToPrimitive;

use crate::{eval::EvalError, value::ValueKind, Eval, Scope, Value};

impl Eval for Expr {
    fn eval(&self, scope: &mut impl Scope) -> Result<Value, EvalError> {
        match &self.kind {
            // AST ERROR
            ExprKind::Invalid => Err(EvalError::InvalidNode(self.id())),

            // BASE VALUES
            ExprKind::None => Ok(Value::None),
            ExprKind::Bool(v) => Ok(Value::Bool(*v)),
            ExprKind::Int(v) => Ok(Value::Int(v.clone())),
            ExprKind::Float(v) => Ok(Value::Float(*v)),
            ExprKind::String(v) => Ok(Value::Str(v.clone())),
            ExprKind::Fn(v) => Ok(Value::Fn(v.clone())),
            ExprKind::Var(id) => match scope.get(id) {
                Some(value) => Ok(value.clone()),
                None => Err(EvalError::UnknownVariable {
                    id: self.id(),
                    name: id.clone(),
                }),
            },

            // BINARY OPS
            ExprKind::Pos(expr) => match expr.eval(scope)? {
                Value::Int(v) => Ok(Value::Int(v)),
                Value::Float(v) => Ok(Value::Float(v)),
                value => Err(EvalError::InvalidUnaryOp {
                    id: self.id(),
                    op: format!("positive"),
                    kind: value.kind(),
                }),
            },
            ExprKind::Neg(expr) => match expr.eval(scope)? {
                Value::Int(v) => Ok(Value::Int(-v)),
                Value::Float(v) => Ok(Value::Float(-v)),
                value => Err(EvalError::InvalidUnaryOp {
                    id: self.id(),
                    op: format!("negative"),
                    kind: value.kind(),
                }),
            },
            ExprKind::Not(expr) => match expr.eval(scope)? {
                Value::Bool(v) => Ok(Value::Bool(!v)),
                value => Err(EvalError::InvalidUnaryOp {
                    id: self.id(),
                    op: format!("not"),
                    kind: value.kind(),
                }),
            },

            // ADD
            ExprKind::Add(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs + rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Float(lhs.to_f64().value() + rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs + rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Float(lhs + rhs.to_f64().value()))
                }

                // strings
                (Value::Str(lhs), Value::None) => Ok(Value::Str(lhs)),
                (Value::Str(lhs), Value::Int(rhs)) => Ok(Value::Str(format!("{lhs}{rhs}"))),
                (Value::Str(lhs), Value::Float(rhs)) => Ok(Value::Str(format!("{lhs}{rhs}"))),
                (Value::Str(lhs), Value::Bool(rhs)) => Ok(Value::Str(format!("{lhs}{rhs}"))),
                (Value::Str(lhs), Value::Str(rhs)) => Ok(Value::Str(format!("{lhs}{rhs}"))),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("add"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // SUBTRACT
            ExprKind::Sub(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs - rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Float(lhs.to_f64().value() - rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs - rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Float(lhs - rhs.to_f64().value()))
                }

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("subtract"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // MULTIPLY
            ExprKind::Mul(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs * rhs)),
                (Value::Int(lhs), Value::Bool(rhs)) => Ok(Value::Int(lhs * rhs as u8)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Float(lhs.to_f64().value() * rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs * rhs)),
                (Value::Float(lhs), Value::Bool(rhs)) => Ok(Value::Float(lhs * rhs as u8 as f64)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Float(lhs * rhs.to_f64().value()))
                }

                // strings
                (Value::Str(lhs), Value::Bool(rhs)) => match rhs {
                    false => Ok(Value::Str(format!(""))),
                    true => Ok(Value::Str(lhs)),
                },
                (Value::Str(lhs), Value::Int(rhs)) => {
                    let (sign, int) = rhs.into_parts();
                    match sign {
                        Sign::Negative => Ok(Value::Str(format!(""))),
                        Sign::Positive => match int.to_usize() {
                            Some(count) => Ok(Value::Str(lhs.repeat(count))),
                            None => Ok(Value::Str(lhs.repeat(usize::MAX))),
                        },
                    }
                }

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("multiply"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // DIVIDE
            ExprKind::Div(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs / rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Float(lhs.to_f64().value() / rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs / rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Float(lhs / rhs.to_f64().value()))
                }

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("divide"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // MODULO
            ExprKind::Mod(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs % rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Float(lhs.to_f64().value() % rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs % rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Float(lhs % rhs.to_f64().value()))
                }

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("modulo"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // POWER
            ExprKind::Pow(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Float(
                    lhs.to_f64().value_ref().powf(rhs.to_f64().value()),
                )),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Float(lhs.to_f64().value_ref().powf(rhs)))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs.powf(rhs))),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Float(lhs.powf(rhs.to_f64().value())))
                }

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("power"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // EQUAL
            ExprKind::Eq(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Bool(lhs.to_f64().value() == rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Bool(lhs == rhs.to_f64().value()))
                }

                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs == rhs)),

                // strings
                (Value::Str(lhs), Value::Str(rhs)) => Ok(Value::Bool(lhs == rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("=="),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // LESS THAN
            ExprKind::Lt(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs.to_f64().value() < rhs)),

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs < rhs.to_f64().value())),

                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs < rhs)),

                // strings
                (Value::Str(lhs), Value::Str(rhs)) => Ok(Value::Bool(lhs < rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("<"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // GREATER THAN
            ExprKind::Gt(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs.to_f64().value() > rhs)),

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs > rhs.to_f64().value())),

                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs > rhs)),

                // strings
                (Value::Str(lhs), Value::Str(rhs)) => Ok(Value::Bool(lhs > rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!(">"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // NOT EQUAL
            ExprKind::NEq(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Bool(lhs.to_f64().value() != rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Bool(lhs != rhs.to_f64().value()))
                }

                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs != rhs)),

                // strings
                (Value::Str(lhs), Value::Str(rhs)) => Ok(Value::Bool(lhs != rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("!="),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // LESS THAN OR EQUAL
            ExprKind::LtEq(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Bool(lhs.to_f64().value() <= rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Bool(lhs <= rhs.to_f64().value()))
                }

                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs <= rhs)),

                // strings
                (Value::Str(lhs), Value::Str(rhs)) => Ok(Value::Bool(lhs <= rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("<="),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // GREATER THAN OR EQUAL
            ExprKind::GtEq(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // integers
                (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::Int(lhs), Value::Float(rhs)) => {
                    Ok(Value::Bool(lhs.to_f64().value() >= rhs))
                }

                // floats
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::Float(lhs), Value::Int(rhs)) => {
                    Ok(Value::Bool(lhs >= rhs.to_f64().value()))
                }

                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs >= rhs)),

                // strings
                (Value::Str(lhs), Value::Str(rhs)) => Ok(Value::Bool(lhs >= rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!(">="),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // AND
            ExprKind::And(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs && rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("and"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // OR
            ExprKind::Or(lhs, rhs) => match (lhs.eval(scope)?, rhs.eval(scope)?) {
                // bools
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs || rhs)),

                // invalid
                (lhs, rhs) => Err(EvalError::InvalidBinaryOp {
                    id: self.id(),
                    op: format!("or"),
                    lhs: lhs.kind(),
                    rhs: rhs.kind(),
                }),
            },

            // TERNARY
            ExprKind::Ternary { cond, pass, fail } => match cond.eval(scope)? {
                // valid conditions
                Value::Bool(true) => pass.eval(scope),
                Value::Bool(false) => fail.eval(scope),

                // invalid
                value => Err(EvalError::InvalidType {
                    id: cond.id(),
                    expect: ValueKind::Bool,
                    found: value.kind(),
                }),
            },

            // WALRUS
            ExprKind::Walrus(lhs, rhs) => {
                let value = rhs.eval(scope)?;
                match &lhs.kind {
                    ExprKind::Var(id) => match scope.get_local_mut(id) {
                        None => {
                            scope.init_local(id, value.clone());
                            Ok(value)
                        }
                        Some(old_value) => {
                            *old_value = value.clone();
                            Ok(value)
                        }
                    },
                    _ => Err(EvalError::InvalidAssignment(lhs.id())),
                }
            }
        }
    }
}
