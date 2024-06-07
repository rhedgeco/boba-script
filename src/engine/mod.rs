mod engine;
mod scope;

pub mod boba;
pub mod error;
pub mod func;
pub mod value;

pub use engine::*;

pub use func::{FuncValue, NativeFunc};
pub use value::Value;
