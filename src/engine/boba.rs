use crate::{engine::Value, Engine};

use super::{value::FuncValue, NativeFunc};

pub fn load_boba_lib<Data>(engine: &mut Engine<Data>) {
    engine.init_const("print", Value::Func(FuncValue::Native(native_print())))
}

pub fn native_print<Data>() -> NativeFunc<Data> {
    NativeFunc::new("print", 1, |values| {
        let message = &values[0];
        println!("{message}");
        Ok(Value::None)
    })
}
