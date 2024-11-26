mod program;

pub mod error;
pub mod scope;
pub mod utils;

pub use program::*;

pub use error::CompileError;
pub use scope::Scope;

#[cfg(test)]
mod test;
