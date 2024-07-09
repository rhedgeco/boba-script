use std::{fmt::Debug, marker::PhantomData};

use crate::{
    ast::{expr, Expr},
    engine::Value,
};

use super::{eval::Evaluate, ops::OpManager, EvalError, ScopeStack};

pub struct Engine<Data> {
    _data: PhantomData<*const Data>,
    vars: ScopeStack,
    ops: OpManager,
}

impl<Data> Debug for Engine<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("_data", &self._data)
            .field("vars", &self.vars)
            .field("ops", &self.ops)
            .finish()
    }
}

impl<Data> Default for Engine<Data> {
    fn default() -> Self {
        Self {
            _data: Default::default(),
            vars: Default::default(),
            ops: Default::default(),
        }
    }
}

impl<Data> Engine<Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ops(&self) -> &OpManager {
        &self.ops
    }

    pub fn vars(&self) -> &ScopeStack {
        &self.vars
    }

    pub fn vars_mut(&mut self) -> &mut ScopeStack {
        &mut self.vars
    }
}

impl<Data: Clone> Engine<Data> {
    pub fn eval(&mut self, item: impl Evaluate<Data>) -> Result<Value, EvalError<Data>> {
        item.eval_with(self)
    }

    pub fn assign(&mut self, lhs: &Expr<Data>, rhs: &Expr<Data>) -> Result<(), EvalError<Data>> {
        let store = self.destructure(lhs, rhs)?;
        for (id, value, data) in store {
            if let Err(_) = self.vars.set(id, value) {
                return Err(EvalError::UnknownVariable {
                    name: id.to_string(),
                    data: data.clone(),
                });
            }
        }

        Ok(())
    }

    pub fn init_assign(
        &mut self,
        lhs: &Expr<Data>,
        rhs: &Expr<Data>,
    ) -> Result<(), EvalError<Data>> {
        let store = self.destructure(lhs, rhs)?;
        for (id, value, _) in store {
            self.vars.init(id, value);
        }
        Ok(())
    }

    fn destructure<'a, 'b>(
        &mut self,
        lhs: &'a Expr<Data>,
        rhs: &'b Expr<Data>,
    ) -> Result<Vec<(&'a str, Value, &'b Data)>, EvalError<Data>> {
        fn recurse<'a, 'b, Data: Clone>(
            lhs: &'a Expr<Data>,
            rhs: &'b Expr<Data>,
            engine: &mut Engine<Data>,
            store: &mut Vec<(&'a str, Value, &'b Data)>,
        ) -> Result<(), EvalError<Data>> {
            match &lhs.kind {
                // if the lhs is a variable, then directly assign to it
                expr::Kind::Var(id) => {
                    let value = engine.eval(rhs)?;
                    store.push((id, value, &rhs.data));
                    Ok(())
                }
                // if the lhs is a tuple, then loop over each inner expr and assign
                expr::Kind::Tuple(lhs_exprs) => match &rhs.kind {
                    expr::Kind::Tuple(rhs_exprs) => match lhs_exprs.len() == rhs_exprs.len() {
                        false => Err(EvalError::DestructureError {
                            data: rhs.data.clone(),
                        }),
                        true => {
                            for (lhs, rhs) in lhs_exprs.iter().zip(rhs_exprs) {
                                recurse(lhs, rhs, engine, store)?;
                            }
                            Ok(())
                        }
                    },
                    _ => match lhs_exprs.len() {
                        1 => recurse(&lhs_exprs[0], rhs, engine, store),
                        _ => Err(EvalError::DestructureError {
                            data: rhs.data.clone(),
                        }),
                    },
                },
                // if the lhs is anything else, then the lhs cannot be assigned to
                _ => {
                    return Err(EvalError::AssignError {
                        data: lhs.data.clone(),
                    })
                }
            }
        }

        // capture all the destructured variables
        let mut store = Vec::new();
        recurse(lhs, rhs, self, &mut store)?;
        Ok(store)
    }
}
