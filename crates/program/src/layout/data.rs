use boba_script_ast::{def::Visibility, path::PathUnion, statement::LocalStatement, Node};
use indexmap::IndexMap;

use crate::indexers::{ClassIndex, FuncIndex, GlobalIndex, ScopeIndex};

#[derive(Debug, Clone, Copy)]
pub struct VisData<T> {
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
pub struct ScopeData {
    pub super_scope: Option<ScopeIndex>,
    pub parent_scope: Option<ScopeIndex>,
    pub defs: IndexMap<String, VisData<DefIndex>>,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub fields: IndexMap<String, VisData<PathUnion>>,
}

#[derive(Debug, Clone)]
pub struct FuncBodyData {
    pub inner_scope: ScopeIndex,
    pub statements: Vec<LocalStatement>,
}

#[derive(Debug, Clone)]
pub struct FuncData {
    pub parent_scope: ScopeIndex,
    pub parameters: IndexMap<String, Node<PathUnion>>,
    pub output: Node<PathUnion>,
    pub body: Option<FuncBodyData>,
}
