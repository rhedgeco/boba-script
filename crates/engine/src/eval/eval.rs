use crate::{Scope, Value};

use super::EvalError;

pub trait Eval {
    fn eval(&self, scope: &mut impl Scope) -> Result<Value, EvalError>;
}
