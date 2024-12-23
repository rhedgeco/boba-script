use boba_script_ast::Node;
use indexmap::IndexMap;

use crate::{
    indexers::{ClassIndex, FuncIndex},
    layout::data::VisData,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolvedValue {
    Any,
    None,
    Bool,
    Int,
    Float,
    String,
    Class(ClassIndex),
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub fields: IndexMap<String, Vec<ResolvedValue>>,
    pub funcs: IndexMap<String, VisData<FuncIndex>>,
}

#[derive(Debug, Clone)]
pub struct FuncData {
    pub inputs: IndexMap<String, Vec<ResolvedValue>>,
    pub output: Vec<Node<ResolvedValue>>,
}
