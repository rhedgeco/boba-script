use std::ops::Deref;

use crate::Engine;

use super::{EvalError, Value};

pub trait Evaluate<Data: Clone> {
    fn eval_with(&self, engine: &mut Engine<Data>) -> Result<Value, EvalError<Data>>;
}

impl<Data: Clone, E: Evaluate<Data>, T: Deref<Target = E>> Evaluate<Data> for T {
    fn eval_with(&self, engine: &mut Engine<Data>) -> Result<Value, EvalError<Data>> {
        self.deref().eval_with(engine)
    }
}
