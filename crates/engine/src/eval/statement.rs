use boba_script_ast::{expr::ExprKind, statement::StatementKind, Node, Statement};

use crate::{eval::EvalError, value::ValueKind, Eval, Scope, Value};

impl Eval for Statement {
    fn eval(&self, scope: &mut impl Scope) -> Result<Value, EvalError> {
        match &self.kind {
            StatementKind::Invalid => Err(EvalError::InvalidNode(self.id())),
            StatementKind::Expr(expr) => expr.eval(scope),
            StatementKind::Assign(lhs, rhs) => {
                let value = rhs.eval(scope)?;
                match &lhs.kind {
                    ExprKind::Var(id) => match scope.get_local_mut(id) {
                        None => {
                            scope.init_local(id, value);
                            Ok(Value::None)
                        }
                        Some(old_value) => {
                            *old_value = value;
                            Ok(Value::None)
                        }
                    },
                    _ => Err(EvalError::InvalidAssignment(lhs.id())),
                }
            }
            StatementKind::If { cond, body } => match cond.eval(scope)? {
                Value::Bool(false) => Ok(Value::None), // do nothing for now
                Value::Bool(true) => {
                    for statement in body {
                        statement.eval(scope)?;
                    }
                    Ok(Value::None)
                }

                value => Err(EvalError::InvalidType {
                    id: cond.id(),
                    expect: ValueKind::Bool,
                    found: value.kind(),
                }),
            },
            StatementKind::While { cond, body } => {
                while match cond.eval(scope)? {
                    Value::Bool(bool) => bool,
                    value => {
                        return Err(EvalError::InvalidType {
                            id: cond.id(),
                            expect: ValueKind::Bool,
                            found: value.kind(),
                        })
                    }
                } {
                    for statement in body {
                        statement.eval(scope)?;
                    }
                }

                Ok(Value::None)
            }
        }
    }
}
