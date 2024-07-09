use crate::{
    engine::{utils, value::ValueKind, EvalError, Value},
    Engine,
};

use super::{Carrier, Expr};

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
                // destructure all the variable assignments
                let store = utils::destructure(lhs, rhs, engine)?;

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
