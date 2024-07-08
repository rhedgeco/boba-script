use derive_more::Display;

use crate::{engine::Value, Engine};

use super::{expr, Carrier, Expr};

pub enum Kind<Data> {
    Expr(Expr<Data>),
    Assign {
        init: bool,
        lhs: Expr<Data>,
        rhs: Expr<Data>,
    },
    While {
        cond: Expr<Data>,
        body: Vec<Statement<Data>>,
    },
}

impl<Data> Kind<Data> {
    pub fn carry(self, data: Data) -> Statement<Data> {
        Statement { kind: self, data }
    }
}

#[derive(Debug, Display, Clone)]
pub enum EvalError<Data> {
    #[display(fmt = "{}", _0)]
    Expr(expr::EvalError<Data>),
    #[display(fmt = "cannot assign to expression")]
    AssignError { data: Data },
    #[display(fmt = "unknown variable '{}'", name)]
    UnknownVariable { name: String, data: Data },
    #[display(fmt = "expected type '{}', found type '{}'", expect, found)]
    UnexpectedType {
        expect: String,
        found: String,
        data: Data,
    },
}

impl<Data> From<expr::EvalError<Data>> for EvalError<Data> {
    fn from(value: expr::EvalError<Data>) -> Self {
        Self::Expr(value)
    }
}

pub struct Statement<Data> {
    pub kind: Kind<Data>,
    pub data: Data,
}

impl<Data: Clone> Statement<Data> {
    pub fn eval(&self, engine: &mut Engine) -> Result<Value, EvalError<Data>> {
        match &self.kind {
            Kind::Expr(expr) => Ok(expr.eval(engine)?),
            Kind::Assign { init, lhs, rhs } => match &lhs.kind {
                expr::Kind::Var(id) => {
                    let value = rhs.eval(engine)?;
                    match init {
                        true => {
                            engine.vars_mut().init(id, value);
                            Ok(Value::None)
                        }
                        false => match engine.vars_mut().set(id, value) {
                            Ok(_) => Ok(Value::None),
                            Err(_) => Err(EvalError::UnknownVariable {
                                name: id.clone(),
                                data: lhs.data().clone(),
                            }),
                        },
                    }
                }
                _ => Err(EvalError::AssignError {
                    data: lhs.data().clone(),
                }),
            },
            Kind::While { cond, body } => loop {
                match cond.eval(engine)? {
                    Value::Bool(true) => (),
                    Value::Bool(false) => break Ok(Value::None),
                    value => {
                        break Err(EvalError::UnexpectedType {
                            expect: "bool".into(),
                            found: value.type_name(),
                            data: cond.data().clone(),
                        })
                    }
                }

                for statement in body {
                    statement.eval(engine)?;
                }
            },
        }
    }
}
