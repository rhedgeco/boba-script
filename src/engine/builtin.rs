use crate::{engine::Value, Engine};

use super::{FuncValue, NativeFunc};

pub fn load_builtins<Data: Clone>(engine: &mut Engine<Data>) {
    engine.init_func(FuncValue::native(native_print()))
}

pub fn native_print<Data>() -> NativeFunc<Data> {
    NativeFunc::new("print", 1, |values| {
        match &values[0] {
            Value::None => println!(""),
            Value::Bool(v) => println!("{v}"),
            Value::Int(v) => println!("{v}"),
            Value::Float(v) => println!("{v}"),
            Value::String(v) => println!("{v}"),
            Value::Func(func) => println!("fn {}", func.ident()),
        }

        Ok(Value::None)
    })
}
