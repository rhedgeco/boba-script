use crate::Engine;

use super::{value::FuncPtr, Value};

pub fn load_into<Source>(engine: &mut Engine<Source>) {
    engine.vars_mut().init_global(
        "print",
        Value::Func(FuncPtr::native(1, |values| {
            println!("{}", values[0]);
            Ok(Value::None)
        })),
    )
}
