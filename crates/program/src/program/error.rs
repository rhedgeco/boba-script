use boba_script_ast::node::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompileError {
    SuperFromRootScope,
    SuperInPath,
    ModuleDoesNotExist,
    ClassDoesNotExist,
    FuncDoesNotExist,
    NotAClass(NodeId),
    NotAFunc(NodeId),
    PrivateModule,
    PrivateClass,
    PrivateField,
    PrivateFunc,
    EmptyPath,
}
