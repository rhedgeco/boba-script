use std::ops::Index;

use indexmap::IndexMap;

use crate::{
    indexers::{ClassIndex, FieldIndex, FuncIndex, InputIndex},
    utils::resolve_class,
    CompileError, ProgramLayout,
};

pub struct Class {
    fields: IndexMap<String, Vec<ClassIndex>>,
}

impl Index<FieldIndex> for Class {
    type Output = [ClassIndex];

    fn index(&self, index: FieldIndex) -> &Self::Output {
        &self.fields[index.raw()]
    }
}

impl Class {
    pub fn get_field(&self, name: impl AsRef<str>) -> Option<&[ClassIndex]> {
        self.fields.get(name.as_ref()).map(|v| v.as_slice())
    }

    pub fn get_field_index(&self, index: FieldIndex) -> Option<&[ClassIndex]> {
        self.fields
            .get_index(index.raw())
            .map(|(_, v)| v.as_slice())
    }

    pub fn get_field_index_of(&self, name: impl AsRef<str>) -> Option<FieldIndex> {
        Some(FieldIndex::from_raw(
            self.fields.get_index_of(name.as_ref())?,
        ))
    }
}

pub struct Func {
    inputs: IndexMap<String, Vec<ClassIndex>>,
    output: Vec<ClassIndex>,
}

impl Index<InputIndex> for Func {
    type Output = [ClassIndex];

    fn index(&self, index: InputIndex) -> &Self::Output {
        &self.inputs[index.raw()]
    }
}

impl Func {
    pub fn output(&self) -> &[ClassIndex] {
        &self.output
    }

    pub fn get_input(&self, name: impl AsRef<str>) -> Option<&[ClassIndex]> {
        self.inputs.get(name.as_ref()).map(|v| v.as_slice())
    }

    pub fn get_input_index(&self, index: FieldIndex) -> Option<&[ClassIndex]> {
        self.inputs
            .get_index(index.raw())
            .map(|(_, v)| v.as_slice())
    }

    pub fn get_input_index_of(&self, name: impl AsRef<str>) -> Option<FieldIndex> {
        Some(FieldIndex::from_raw(
            self.inputs.get_index_of(name.as_ref())?,
        ))
    }
}

pub struct Program {
    classes: Vec<Class>,
    funcs: Vec<Func>,
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

        // return errors if there are any
        if !errors.is_empty() {
            return Err(errors);
        }

        // resolve all program class data
        for class_data in layout.classes() {
            // resolve all class fields
            let mut fields = IndexMap::new();
            for (name, union) in &class_data.fields {
                let mut types = Vec::new();
                for path in &union.data.paths {
                    match resolve_class(layout, class_data.parent_scope, path.iter()) {
                        Ok(class_index) => types.push(class_index),
                        Err(error) => errors.push(error),
                    }
                }
                fields.insert(name.to_string(), types);
            }

            // build the class and append it
            classes.push(Class { fields })
        }

        // resolve all program func data
        for func_data in layout.funcs() {
            // resolve the output
            let mut output = Vec::new();
            for path in &func_data.output.paths {
                match resolve_class(layout, func_data.parent_scope, path.iter()) {
                    Ok(class_index) => output.push(class_index),
                    Err(error) => errors.push(error),
                }
            }

            // resolve all inputs
            let mut inputs = IndexMap::new();
            for (name, union) in &func_data.inputs {
                let mut types = Vec::new();
                for path in &union.paths {
                    match resolve_class(layout, func_data.parent_scope, path.iter()) {
                        Ok(class_index) => types.push(class_index),
                        Err(error) => errors.push(error),
                    }
                }
                inputs.insert(name.to_string(), types);
            }

            // build the function and append it
            funcs.push(Func { inputs, output })
        }

        // if there are errors, return those
        if !errors.is_empty() {
            return Err(errors);
        }

        // otherwise, return the compiled program
        Ok(Program { classes, funcs })
    }
}
