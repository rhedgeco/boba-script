use boba_script_ast::node::NodeId;
use derive_more::derive::Display;
use thiserror::Error;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum ResolveError {
    #[display("root module has no parent module")]
    SuperFromRoot(NodeId),
    #[display("super keyword is only valid at the start of a path")]
    SuperInPath(NodeId),
    #[display("identifier could not be found")]
    IdentNotFound(NodeId),
    #[display("this identifier is private member and cannot be accessed directly")]
    PrivateIdent(NodeId),
    #[display("identifier is not a global")]
    NotAGlobal(NodeId),
    #[display("identifier is not a module")]
    NotAModule(NodeId),
    #[display("identifier is not a class")]
    NotAClass(NodeId),
    #[display("identifier is not a function")]
    NotAFunc(NodeId),
    #[display("path cannot be empty")]
    EmptyPath,
}
