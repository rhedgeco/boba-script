use std::ops::{Deref, Index};

use boba_script_ast::Node;
use indexmap::IndexMap;

use crate::{
    indexers::{ClassIndex, FuncIndex},
    layout::data::Vis,
    ProgramLayout,
};

use super::{
    data::{ResolvedClass, ResolvedFunc},
    utils::resolve_value,
    ResolveError,
};

#[derive(Debug, Clone)]
pub struct ResolvedProgram {
    errors: Vec<ResolveError>,
    classes: Vec<ResolvedClass>,
    funcs: Vec<ResolvedFunc>,
}

impl Index<ClassIndex> for ResolvedProgram {
    type Output = ResolvedClass;

    fn index(&self, index: ClassIndex) -> &Self::Output {
        &self.classes[index.raw()]
    }
}

impl Index<FuncIndex> for ResolvedProgram {
    type Output = ResolvedFunc;

    fn index(&self, index: FuncIndex) -> &Self::Output {
        &self.funcs[index.raw()]
    }
}

impl ResolvedProgram {
    pub fn errors(&self) -> &[ResolveError] {
        &self.errors
    }

    pub fn classes(&self) -> &[ResolvedClass] {
        &self.classes
    }

    pub fn funcs(&self) -> &[ResolvedFunc] {
        &self.funcs
    }

    pub fn get_class(&self, index: ClassIndex) -> Option<&ResolvedClass> {
        self.classes.get(index.raw())
    }

    pub fn get_func(&self, index: FuncIndex) -> Option<&ResolvedFunc> {
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
            use crate::layout::data::DefIndex as D;
            let inner_scope = &layout[class_data.inner_scope];
            let funcs = inner_scope
                .defs
                .iter()
                .filter_map(|(name, vis)| match vis.data.deref() {
                    D::Func(func_index) => Some((
                        name.clone(),
                        Vis {
                            vis: vis.vis.clone(),
                            data: Node {
                                id: vis.data.id,
                                item: func_index.clone(),
                            },
                        },
                    )),
                    _ => None,
                })
                .collect();

            // build the class and append it
            classes.push(ResolvedClass { fields, funcs })
        }

        // resolve all program func data
        for func_data in layout.funcs() {
            // resolve the output
            let mut output = Vec::new();
            for concrete in &func_data.output.types {
                match resolve_value(layout, func_data.parent_scope, concrete) {
                    Ok(class_index) => output.push(class_index),
                    Err(error) => errors.push(error),
                }
            }

            // resolve all inputs
            let mut inputs = IndexMap::new();
            for (name, union) in &func_data.parameters {
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
            funcs.push(ResolvedFunc {
                inputs,
                output,
                body: vec![],
            })
        }

        Self {
            errors,
            classes,
            funcs,
        }
    }
}
