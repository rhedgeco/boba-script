use boba_script_ast::node::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayoutError {
    ModuleAlreadyExists { insert: NodeId, found: NodeId },
    ClassAlreadyExists { insert: NodeId, found: NodeId },
    FuncAlreadyExists { insert: NodeId, found: NodeId },
    Unimplemented { id: NodeId, message: &'static str },
}
