use std::ops::Index;

use boba_script_ast::Node;
use indexmap::IndexMap;

use crate::{
    indexers::{ClassIndex, FuncIndex},
    layout::VisData,
    ProgramLayout,
};

use super::{utils::resolve_value, ResolveError};

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

#[derive(Debug, Clone)]
pub struct ResolvedProgram {
    errors: Vec<ResolveError>,
    classes: Vec<ClassData>,
    funcs: Vec<FuncData>,
}

impl Index<ClassIndex> for ResolvedProgram {
    type Output = ClassData;

    fn index(&self, index: ClassIndex) -> &Self::Output {
        &self.classes[index.raw()]
    }
}

impl Index<FuncIndex> for ResolvedProgram {
    type Output = FuncData;

    fn index(&self, index: FuncIndex) -> &Self::Output {
        &self.funcs[index.raw()]
    }
}

impl ResolvedProgram {
    pub fn errors(&self) -> &[ResolveError] {
        &self.errors
    }

    pub fn classes(&self) -> &[ClassData] {
        &self.classes
    }

    pub fn funcs(&self) -> &[FuncData] {
        &self.funcs
    }

    pub fn get_class(&self, index: ClassIndex) -> Option<&ClassData> {
        self.classes.get(index.raw())
    }

    pub fn get_func(&self, index: FuncIndex) -> Option<&FuncData> {
        self.funcs.get(index.raw())
    }

    pub fn resolve(layout: &ProgramLayout) -> Self {
        // build vecs up front
        let mut errors = Vec::new();
        let mut classes = Vec::new();
        let mut funcs = Vec::new();

        // resolve all program class data
        for class_data in layout.classes() {
            // resolve all class fields
            let mut fields = IndexMap::new();
            for (name, union) in &class_data.fields {
                let mut types = Vec::new();
                for concrete in &union.data.types {
                    match resolve_value(layout, class_data.parent_scope, concrete) {
                        Ok(value_kind) => types.push(value_kind),
                        Err(error) => errors.push(error),
                    }
                }
                fields.insert(name.to_string(), types);
            }

            // copy all internal function indices
            let inner_scope = &layout[class_data.inner_scope];
            let funcs = inner_scope.funcs.clone();

            // build the class and append it
            classes.push(ClassData { fields, funcs })
        }

        // resolve all program func data
        for func_data in layout.funcs() {
            // resolve the output
            let mut output = Vec::new();
            for concrete in &func_data.output.types {
                match resolve_value(layout, func_data.parent_scope, concrete) {
                    Err(error) => errors.push(error),
                    Ok(class_index) => output.push(Node {
                        id: concrete.id,
                        item: class_index,
                    }),
                }
            }

            // resolve all inputs
            let mut inputs = IndexMap::new();
            for (name, union) in &func_data.inputs {
                let mut types = Vec::new();
                for concrete in &union.types {
                    match resolve_value(layout, func_data.parent_scope, concrete) {
                        Ok(class_index) => types.push(class_index),
                        Err(error) => errors.push(error),
                    }
                }
                inputs.insert(name.to_string(), types);
            }

            // build the function and append it
            funcs.push(FuncData { inputs, output })
        }

        Self {
            errors,
            classes,
            funcs,
        }
    }
}
