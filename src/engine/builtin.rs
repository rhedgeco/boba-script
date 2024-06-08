use crate::{engine::Value, Engine};

use super::{FuncValue, NativeFunc};

pub fn load_builtins<Data: Clone>(engine: &mut Engine<Data>) {
    load_print(engine);
}

pub fn load_print<Data: Clone>(engine: &mut Engine<Data>) {
    engine.init_const(
        "print",
        Value::Func(FuncValue::native(NativeFunc::new(1, |values| {
            match &values[0] {
                Value::None => println!(""),
                Value::String(v) => println!("{v}"),
                value => println!("{value}"),
            }
            Ok(Value::None)
        }))),
    );
}
