use boba_script_ast::{def::Visibility, path::PathUnion, statement::LocalStatement, Node};
use indexmap::IndexMap;

use crate::indexers::{ClassIndex, FuncIndex, GlobalIndex, ScopeIndex};

#[derive(Debug, Clone, Copy)]
pub struct VisLayout<T> {
    pub vis: Node<Visibility>,
    pub data: Node<T>,
}

#[derive(Debug, Clone, Copy)]
pub enum DefIndex {
    Global(GlobalIndex),
    Module(ScopeIndex),
    Class(ClassIndex),
    Func(FuncIndex),
}

#[derive(Debug, Clone)]
pub struct ScopeLayout {
    pub super_scope: Option<ScopeIndex>,
    pub parent_scope: Option<ScopeIndex>,
    pub defs: IndexMap<String, VisLayout<DefIndex>>,
}

#[derive(Debug, Clone)]
pub struct ClassLayout {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub fields: IndexMap<String, VisLayout<PathUnion>>,
}

#[derive(Debug, Clone)]
pub struct FuncLayout {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub parameters: IndexMap<String, Node<PathUnion>>,
    pub output: Node<PathUnion>,
    pub body: Vec<LocalStatement>,
}
