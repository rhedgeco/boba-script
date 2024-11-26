mod program;

pub mod error;
pub mod utils;

pub use program::*;

pub use error::CompileError;

#[cfg(test)]
mod test;
