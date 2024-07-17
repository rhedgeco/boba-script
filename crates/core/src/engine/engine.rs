use crate::{
    ast::{expr::ExprNode, node::EvalNode, Expr, Node},
    engine::Value,
};

use super::{builtins, ops::OpManager, value::ValueStore, EvalError};

pub struct Engine<Source> {
    values: ValueStore<Source>,
    ops: OpManager<Source>,
}

impl<Source> Default for Engine<Source> {
    fn default() -> Self {
        let mut engine = Self::empty();
        builtins::load_into(&mut engine);
        engine
    }
}

impl<Source> Engine<Source> {
    pub fn empty() -> Self {
        Self {
            values: Default::default(),
            ops: Default::default(),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn ops(&self) -> &OpManager<Source> {
        &self.ops
    }

    pub fn vars(&self) -> &ValueStore<Source> {
        &self.values
    }

    pub fn vars_mut(&mut self) -> &mut ValueStore<Source> {
        &mut self.values
    }
}

impl<Source: Clone> Engine<Source> {
    pub fn eval<T: EvalNode<Source>>(
        &mut self,
        node: impl AsRef<Node<T, Source>>,
    ) -> Result<Value<Source>, EvalError<Source>> {
        T::eval_node(node.as_ref(), self)
    }

    pub fn assign(
        &mut self,
        lhs: &ExprNode<Source>,
        rhs: &ExprNode<Source>,
    ) -> Result<(), EvalError<Source>> {
        let store = self.destructure(lhs, rhs)?;
        for (id, value, source) in store {
            if let Err(_) = self.values.set(id, value) {
                return Err(EvalError::UnknownVariable {
                    name: id.to_string(),
                    source: source.clone(),
                });
            }
        }

        Ok(())
    }

    pub fn init_assign(
        &mut self,
        lhs: &ExprNode<Source>,
        rhs: &ExprNode<Source>,
    ) -> Result<(), EvalError<Source>> {
        let store = self.destructure(lhs, rhs)?;
        for (id, value, _) in store {
            self.values.init_local(id, value);
        }
        Ok(())
    }

    fn destructure<'a, 'b>(
        &mut self,
        lhs: &'a ExprNode<Source>,
        rhs: &'b ExprNode<Source>,
    ) -> Result<Vec<(&'a str, Value<Source>, &'b Source)>, EvalError<Source>> {
        fn recurse<'a, 'b, Source: Clone>(
            lhs: &'a ExprNode<Source>,
            rhs: &'b ExprNode<Source>,
            engine: &mut Engine<Source>,
            store: &mut Vec<(&'a str, Value<Source>, &'b Source)>,
        ) -> Result<(), EvalError<Source>> {
            match &lhs.item {
                // if the lhs is a variable, then directly assign to it
                Expr::Var(id) => {
                    let value = engine.eval(rhs)?;
                    store.push((id, value, &rhs.source));
                    Ok(())
                }
                // if the lhs is a tuple, then loop over each inner expr and assign
                Expr::Tuple(lhs_exprs) => match &rhs.item {
                    Expr::Tuple(rhs_exprs) => match lhs_exprs.len() == rhs_exprs.len() {
                        false => Err(EvalError::InvalidTupleSize {
                            lhs_count: lhs_exprs.len(),
                            rhs_count: rhs_exprs.len(),
                            lhs_source: lhs.source.clone(),
                            rhs_source: rhs.source.clone(),
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
                        _ => Err(EvalError::InvalidTupleDestructure {
                            lhs_count: lhs_exprs.len(),
                            lhs_source: lhs.source.clone(),
                            rhs_source: rhs.source.clone(),
                        }),
                    },
                },
                // if the lhs is anything else, then the lhs cannot be assigned to
                _ => {
                    return Err(EvalError::InvalidAssign {
                        source: lhs.source.clone(),
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
