use boba_script_ast::node::NodeId;
use derive_more::derive::Display;
use thiserror::Error;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum LayoutError {
    #[display("Duplicate Ident: identifiers in the same scope level must be unique")]
    DuplicateIdent { first: NodeId, second: NodeId },
    #[display("Unimplemented: {message}")]
    Unimplemented { id: NodeId, message: &'static str },
}
