use crate::{engine::Value, Engine};

use super::{FuncValue, NativeFunc};

pub fn load_builtins<Data: Clone>(engine: &mut Engine<Data>) {
    engine.init_func(FuncValue::native(native_print()))
}

pub fn native_print<Data>() -> NativeFunc<Data> {
    NativeFunc::new("print", 1, |values| {
        let message = &values[0];
        println!("{message}");
        Ok(Value::None)
    })
}
