use crate::{
    ast::{expr, Carrier, Expr},
    engine::{EvalError, Value},
    Engine,
};

pub fn destructure<'a, 'b, Data: Clone>(
    lhs: &'a Expr<Data>,
    rhs: &'b Expr<Data>,
    engine: &mut Engine,
) -> Result<Vec<(&'a str, Value, &'b Data)>, EvalError<Data>> {
    fn destructure_recurse<'a, 'b, Data: Clone>(
        lhs: &'a Expr<Data>,
        rhs: &'b Expr<Data>,
        engine: &mut Engine,
        store: &mut Vec<(&'a str, Value, &'b Data)>,
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
                expr::Kind::Tuple(rhs_exprs) => match lhs_exprs.len() == rhs_exprs.len() {
                    false => Err(EvalError::DestructureError {
                        data: rhs.data().clone(),
                    }),
                    true => {
                        for (lhs, rhs) in lhs_exprs.iter().zip(rhs_exprs) {
                            destructure_recurse(lhs, rhs, engine, store)?;
                        }
                        Ok(())
                    }
                },
                _ => match lhs_exprs.len() {
                    1 => destructure_recurse(&lhs_exprs[0], rhs, engine, store),
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

    let mut store = Vec::new();
    destructure_recurse(lhs, rhs, engine, &mut store)?;
    Ok(store)
}
