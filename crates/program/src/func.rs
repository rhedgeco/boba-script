use crate::value::{StackValue, Store};

pub type NativeFunc = fn(&[StackValue], &mut dyn Store) -> Option<StackValue>;

#[derive(Debug, Clone, Copy)]
pub enum FuncCaller {
    Native(NativeFunc),
}
