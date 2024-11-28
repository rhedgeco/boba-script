use std::ops::Index;

use indexmap::IndexMap;

use crate::{
    func::FuncCaller,
    indexers::{ClassIndex, FieldIndex, FuncIndex, InputIndex},
    program::utils::resolve_value,
    ProgramLayout,
};

use super::{CompileError, Scope};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueKind {
    Any,
    None,
    Bool,
    Int,
    Float,
    String,
    Class(ClassIndex),
}

pub struct Class {
    fields: IndexMap<String, Vec<ValueKind>>,
    scope: Scope,
}

impl Index<FieldIndex> for Class {
    type Output = [ValueKind];

    fn index(&self, index: FieldIndex) -> &Self::Output {
        &self.fields[index.raw()]
    }
}

impl Class {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn fields(&self) -> impl Iterator<Item = &[ValueKind]> {
        self.fields.values().map(|v| v.as_slice())
    }

    pub fn get_field(&self, name: impl AsRef<str>) -> Option<&[ValueKind]> {
        self.fields.get(name.as_ref()).map(|v| v.as_slice())
    }

    pub fn get_field_index(&self, index: FieldIndex) -> Option<&[ValueKind]> {
        self.fields
            .get_index(index.raw())
            .map(|(_, v)| v.as_slice())
    }

    pub fn get_field_index_of(&self, name: impl AsRef<str>) -> Option<FieldIndex> {
        Some(FieldIndex::new(self.fields.get_index_of(name.as_ref())?))
    }
}

pub struct Func {
    caller: FuncCaller,
    inputs: IndexMap<String, Vec<ValueKind>>,
    output: Vec<ValueKind>,
    scope: Scope,
}

impl Index<InputIndex> for Func {
    type Output = [ValueKind];

    fn index(&self, index: InputIndex) -> &Self::Output {
        &self.inputs[index.raw()]
    }
}

impl Func {
    pub fn caller(&self) -> FuncCaller {
        self.caller
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn output(&self) -> &[ValueKind] {
        &self.output
    }

    pub fn inputs(&self) -> impl Iterator<Item = &[ValueKind]> {
        self.inputs.values().map(|v| v.as_slice())
    }

    pub fn get_input(&self, name: impl AsRef<str>) -> Option<&[ValueKind]> {
        self.inputs.get(name.as_ref()).map(|v| v.as_slice())
    }

    pub fn get_input_index(&self, index: FieldIndex) -> Option<&[ValueKind]> {
        self.inputs
            .get_index(index.raw())
            .map(|(_, v)| v.as_slice())
    }

    pub fn get_input_index_of(&self, name: impl AsRef<str>) -> Option<FieldIndex> {
        Some(FieldIndex::new(self.inputs.get_index_of(name.as_ref())?))
    }
}

pub struct Program {
    classes: Vec<Class>,
    funcs: Vec<Func>,
    scope: Scope,
}

impl Index<ClassIndex> for Program {
    type Output = Class;

    fn index(&self, index: ClassIndex) -> &Self::Output {
        &self.classes[index.raw()]
    }
}

impl Index<FuncIndex> for Program {
    type Output = Func;

    fn index(&self, index: FuncIndex) -> &Self::Output {
        &self.funcs[index.raw()]
    }
}

impl Program {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn get_class(&self, index: ClassIndex) -> Option<&Class> {
        self.classes.get(index.raw())
    }

    pub fn get_func(&self, index: FuncIndex) -> Option<&Func> {
        self.funcs.get(index.raw())
    }

    pub fn compile(layout: &ProgramLayout) -> Result<Self, Vec<CompileError>> {
        // build vecs up front
        let mut errors = Vec::new();
        let mut classes = Vec::new();
        let mut funcs = Vec::new();

        // if the layout is empty, return an empty program
        let Some(root_scope) = layout.get_root_scope() else {
            return Ok(Self {
                classes,
                funcs,
                scope: Scope::empty(),
            });
        };

        // compile the root scope
        let scope = Scope::compile(layout, root_scope);

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

            // compile the class scope
            let scope = Scope::compile(layout, class_data.inner_scope);

            // build the class and append it
            classes.push(Class { fields, scope })
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

            // compile the function scope
            let scope = Scope::compile(layout, func_data.inner_scope);

            // build the function and append it
            funcs.push(Func {
                caller: func_data.caller,
                inputs,
                output,
                scope,
            })
        }

        // return errors is there are any
        if !errors.is_empty() {
            return Err(errors);
        }

        // otherwise, return the compiled program
        Ok(Program {
            classes,
            funcs,
            scope,
        })
    }
}
