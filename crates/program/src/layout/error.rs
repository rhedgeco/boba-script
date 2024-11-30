use boba_script_ast::node::NodeId;
use derive_more::derive::Display;
use thiserror::Error;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum LayoutError {
    #[display("Duplicate Module: module names in the same scope level must be unique")]
    DuplicateModule { first: NodeId, second: NodeId },
    #[display("Duplicate Class: class names in the same scope level must be unique")]
    DuplicateClass { first: NodeId, second: NodeId },
    #[display("Duplicate Func: func names in the same scope level must be unique")]
    DuplicateFunc { first: NodeId, second: NodeId },
    #[display("Unimplemented: {message}")]
    Unimplemented { id: NodeId, message: &'static str },
}
