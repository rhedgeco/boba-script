use crate::{
    engine::{eval::Evaluate, value::ValueKind, EvalError, Value},
    Engine,
};

use super::Expr;

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

impl<Data: Clone> Evaluate<Data> for Statement<Data> {
    fn eval_with(&self, engine: &mut Engine<Data>) -> Result<Value, EvalError<Data>> {
        match &self.kind {
            Kind::Expr { expr, closed } => {
                let value = engine.eval(expr)?;
                match closed {
                    true => Ok(Value::None),
                    false => Ok(value),
                }
            }
            Kind::Assign { init, lhs, rhs } => {
                match init {
                    false => engine.assign(lhs, rhs)?,
                    true => engine.init_assign(lhs, rhs)?,
                }

                Ok(Value::None)
            }
            Kind::While { cond, body } => loop {
                match engine.eval(cond)? {
                    Value::Bool(true) => (),
                    Value::Bool(false) => break Ok(Value::None),
                    value => {
                        break Err(EvalError::UnexpectedType {
                            expect: ValueKind::Bool,
                            found: value.kind(),
                            data: cond.data.clone(),
                        })
                    }
                }

                for statement in body {
                    engine.eval(statement)?;
                }
            },
        }
    }
}
