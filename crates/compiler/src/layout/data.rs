use boba_script_ast::{
    statement::LocalStatement,
    typ::{TypePath, TypeUnion},
    Node, Visibility,
};
use indexmap::IndexMap;

use crate::indexers::{ClassIndex, FuncIndex, GlobalIndex, ScopeIndex};

#[derive(Debug, Clone, Copy)]
pub struct Vis<T> {
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
    pub defs: IndexMap<String, Vis<DefIndex>>,
}

#[derive(Debug, Clone)]
pub struct ClassLayout {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub fields: IndexMap<String, Vis<TypeUnion<TypePath>>>,
}

#[derive(Debug, Clone)]
pub struct FuncLayout {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub parameters: IndexMap<String, Node<TypeUnion<TypePath>>>,
    pub output: Node<TypeUnion<TypePath>>,
    pub body: Vec<LocalStatement<TypePath>>,
}
