use crate::{
    engine::{value::ValueKind, EvalError, Value},
    Engine,
};

use super::{expr, Carrier, Expr};

pub enum Kind<Data> {
    Expr {
        expr: Expr<Data>,
        closed: bool,
    },
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

pub struct Statement<Data> {
    pub kind: Kind<Data>,
    pub data: Data,
}

impl<Data: Clone> Statement<Data> {
    pub fn eval(&self, engine: &mut Engine) -> Result<Value, EvalError<Data>> {
        match &self.kind {
            Kind::Expr { expr, closed } => {
                let value = expr.eval(engine)?;
                match closed {
                    true => Ok(Value::None),
                    false => Ok(value),
                }
            }
            Kind::Assign { init, lhs, rhs } => {
                fn assign<'a, Data: Clone>(
                    lhs: &'a Expr<Data>,
                    rhs: &'a Expr<Data>,
                    engine: &mut Engine,
                    store: &mut Vec<(&'a str, Value, &'a Data)>,
                ) -> Result<(), EvalError<Data>> {
                    match &lhs.kind {
                        // if the lhs is a variable, then directly assign to it
                        expr::Kind::Var(id) => {
                            let value = rhs.eval(engine)?;
                            store.push((id, value, rhs.data()));
                            Ok(())
                        }
                        // if the lhs is a tuple, then loop over each inner expr and assign
                        expr::Kind::Tuple(lhs_exprs) => match &rhs.kind {
                            expr::Kind::Tuple(rhs_exprs) => {
                                match lhs_exprs.len() == rhs_exprs.len() {
                                    false => Err(EvalError::DestructureError {
                                        data: rhs.data().clone(),
                                    }),
                                    true => {
                                        for (lhs, rhs) in lhs_exprs.iter().zip(rhs_exprs) {
                                            assign(lhs, rhs, engine, store)?;
                                        }
                                        Ok(())
                                    }
                                }
                            }
                            _ => match lhs_exprs.len() {
                                1 => assign(&lhs_exprs[0], rhs, engine, store),
                                _ => Err(EvalError::DestructureError {
                                    data: rhs.data().clone(),
                                }),
                            },
                        },
                        // if the lhs is anything else, then the lhs cannot be assigned to
                        _ => {
                            return Err(EvalError::AssignError {
                                data: lhs.data().clone(),
                            })
                        }
                    }
                }

                // collect all the variable assignments
                let mut store = Vec::new();
                assign(lhs, rhs, engine, &mut store)?;

                // then assign each value in the engine
                match init {
                    true => {
                        for (id, value, _) in store {
                            engine.vars_mut().init(id, value);
                        }
                    }
                    false => {
                        for (id, value, data) in store {
                            match engine.vars_mut().set(&id, value) {
                                Ok(_) => (),
                                Err(_) => {
                                    return Err(EvalError::UnknownVariable {
                                        name: id.to_string(),
                                        data: data.clone(),
                                    })
                                }
                            }
                        }
                    }
                }

                // and return none
                Ok(Value::None)
            }
            Kind::While { cond, body } => loop {
                match cond.eval(engine)? {
                    Value::Bool(true) => (),
                    Value::Bool(false) => break Ok(Value::None),
                    value => {
                        break Err(EvalError::UnexpectedType {
                            expect: ValueKind::Bool,
                            found: value.kind(),
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
