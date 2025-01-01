use boba_script_ast::Node;
use indexmap::IndexMap;

use crate::{
    indexers::{ClassIndex, FuncIndex},
    layout::data::VisLayout,
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
pub struct ResolvedClass {
    pub fields: IndexMap<String, Vec<ResolvedValue>>,
    pub funcs: IndexMap<String, VisLayout<FuncIndex>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedFunc {
    pub inputs: IndexMap<String, Vec<ResolvedValue>>,
    pub output: Vec<ResolvedValue>,
    pub body: Vec<Node<ResolvedStatement>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedStatement {
    _action: (),
}
