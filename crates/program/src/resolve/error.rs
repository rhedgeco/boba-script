use boba_script_ast::node::NodeId;
use derive_more::derive::Display;
use thiserror::Error;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum ResolveError {
    #[display("root module has no parent module")]
    SuperFromRoot(NodeId),
    #[display("super keyword is only valid at the start of a path")]
    SuperInPath(NodeId),
    #[display("module name could not be found in target module")]
    ModuleNotFound(NodeId),
    #[display("class name could not be found in target module")]
    ClassNotFound(NodeId),
    #[display("function name could not be found in target module")]
    FuncNotFound(NodeId),
    #[display("this module is private member and cannot be accessed directly")]
    PrivateModule(NodeId),
    #[display("this class is private member and cannot be accessed directly")]
    PrivateClass(NodeId),
    #[display("this function is private member and cannot be accessed directly")]
    PrivateFunc(NodeId),
    #[display("this field is private member and cannot be accessed directly")]
    PrivateField(NodeId),
    #[display("this item is not a class")]
    NotAClass(NodeId),
    #[display("this item is not a function")]
    NotAFunc(NodeId),
    #[display("path cannot be empty")]
    EmptyPath,
}
