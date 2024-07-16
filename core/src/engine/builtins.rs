use crate::Engine;

use super::{value::func::NativeFunc, Value};

pub fn load_into<Source>(engine: &mut Engine<Source>) {
    engine.vars_mut().init(
        "print",
        Value::Func(NativeFunc::new(1, |values| {
            println!("{}", values[0]);
            Ok(Value::None)
        })),
    )
}
