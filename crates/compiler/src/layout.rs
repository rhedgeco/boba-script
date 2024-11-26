use std::{
    ops::{Deref, Index},
    slice::Iter,
};

use boba_script_ast::{def::Visibility, Class, Definition, Func, Module, Node, Union};
use indexmap::IndexMap;

use crate::indexers::{ClassIndex, FuncIndex, ScopeIndex};

pub struct ScopeData {
    pub super_scope: Option<ScopeIndex>,
    pub parent_scope: Option<ScopeIndex>,
    pub modules: IndexMap<String, ScopeIndex>,
    pub classes: IndexMap<String, ClassIndex>,
    pub funcs: IndexMap<String, FuncIndex>,
    _private: (),
}

pub struct ClassData {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub fields: IndexMap<String, TypeUnion>,
    _private: (),
}

pub struct FuncData {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub inputs: IndexMap<String, TypeUnion>,
    pub output: TypeUnion,
    _private: (),
}

pub struct TypeUnion {
    pub paths: Vec<Vec<String>>,
    _private: (),
}

pub struct ProgramLayout {
    scopes: Vec<ScopeData>,
    classes: Vec<ClassData>,
    funcs: Vec<FuncData>,
}

impl Index<ScopeIndex> for ProgramLayout {
    type Output = ScopeData;

    fn index(&self, index: ScopeIndex) -> &Self::Output {
        &self.scopes[index.raw()]
    }
}

impl Index<ClassIndex> for ProgramLayout {
    type Output = ClassData;

    fn index(&self, index: ClassIndex) -> &Self::Output {
        &self.classes[index.raw()]
    }
}

impl Index<FuncIndex> for ProgramLayout {
    type Output = FuncData;

    fn index(&self, index: FuncIndex) -> &Self::Output {
        &self.funcs[index.raw()]
    }
}

mod private {
    use super::*;
    use std::ops::IndexMut;

    impl IndexMut<ScopeIndex> for ProgramLayout {
        fn index_mut(&mut self, index: ScopeIndex) -> &mut Self::Output {
            &mut self.scopes[index.raw()]
        }
    }

    impl IndexMut<ClassIndex> for ProgramLayout {
        fn index_mut(&mut self, index: ClassIndex) -> &mut Self::Output {
            &mut self.classes[index.raw()]
        }
    }

    impl IndexMut<FuncIndex> for ProgramLayout {
        fn index_mut(&mut self, index: FuncIndex) -> &mut Self::Output {
            &mut self.funcs[index.raw()]
        }
    }
}

impl ProgramLayout {
    pub fn scopes(&self) -> Iter<ScopeData> {
        self.scopes.iter()
    }

    pub fn classes(&self) -> Iter<ClassData> {
        self.classes.iter()
    }

    pub fn funcs(&self) -> Iter<FuncData> {
        self.funcs.iter()
    }

    pub fn get_scope(&self, index: ScopeIndex) -> Option<&ScopeData> {
        self.scopes.get(index.raw())
    }

    pub fn get_class(&self, index: ClassIndex) -> Option<&ClassData> {
        self.classes.get(index.raw())
    }

    pub fn get_func(&self, index: FuncIndex) -> Option<&FuncData> {
        self.funcs.get(index.raw())
    }

    pub fn insert_module_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        module: &Node<Module>,
    ) -> Option<ScopeIndex> {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // try to insert the next module index into the parent scope
        use indexmap::map::Entry as E;
        let new_scope = ScopeIndex::from_raw(self.scopes.len());
        match self[parent_scope].modules.entry(name.to_string()) {
            E::Vacant(entry) => entry.insert(new_scope),
            E::Occupied(_) => return None,
        };

        // get the super scope of the parent
        let super_scope = self[parent_scope].super_scope;

        // build the scope data
        self.scopes.push(ScopeData {
            super_scope,
            parent_scope: None, // module scopes have no parent
            modules: Default::default(),
            classes: Default::default(),
            funcs: Default::default(),
            _private: (),
        });

        // insert all definitions into the new scope
        for def in module.defs.iter() {
            self.insert_definition_into(new_scope, def);
        }

        // return the index of the new scope
        Some(new_scope)
    }

    pub fn insert_class_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        class: &Node<Class>,
    ) -> Option<ClassIndex> {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // try to insert the next class index into the parent scope
        use indexmap::map::Entry as E;
        let new_class = ClassIndex::from_raw(self.classes.len());
        match self[parent_scope].classes.entry(name.to_string()) {
            E::Vacant(entry) => entry.insert(new_class),
            E::Occupied(_) => return None,
        };

        // build all the class fields
        let mut fields = IndexMap::new();
        for field in class.fields.iter() {
            let union = Self::build_union(&field.ty);
            fields.insert(field.name.to_string(), union);
        }

        // get the super scope of the parent
        let super_scope = self[parent_scope].super_scope;

        // build the inner scope and class data
        let inner_scope = ScopeIndex::from_raw(self.scopes.len());
        self.scopes.push(ScopeData {
            super_scope,
            parent_scope: Some(parent_scope),
            modules: Default::default(),
            classes: Default::default(),
            funcs: Default::default(),
            _private: (),
        });
        self.classes.push(ClassData {
            parent_scope,
            inner_scope,
            fields,
            _private: (),
        });

        // insert all definitions into the new scope
        for def in class.defs.iter() {
            self.insert_definition_into(inner_scope, def);
        }

        // return the index of the new class
        Some(new_class)
    }

    pub fn insert_func_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        func: &Node<Func>,
    ) -> Option<FuncIndex> {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // try to insert the next func index into the parent scope
        use indexmap::map::Entry as E;
        let new_func = FuncIndex::from_raw(self.funcs.len());
        match self[parent_scope].funcs.entry(name.to_string()) {
            E::Vacant(entry) => entry.insert(new_func),
            E::Occupied(_) => return None,
        };

        // build the func output and inputs
        let output = Self::build_union(&func.output);
        let mut inputs = IndexMap::new();
        for field in func.inputs.iter() {
            let ty = Self::build_union(&field.ty);
            inputs.insert(field.name.to_string(), ty);
        }

        // get the super scope of the parent
        let super_scope = self[parent_scope].super_scope;

        // build the inner scope and func data
        let inner_scope = ScopeIndex::from_raw(self.scopes.len());
        self.scopes.push(ScopeData {
            super_scope,
            parent_scope: Some(parent_scope),
            modules: Default::default(),
            classes: Default::default(),
            funcs: Default::default(),
            _private: (),
        });
        self.funcs.push(FuncData {
            parent_scope,
            inputs,
            output,
            inner_scope,
            _private: (),
        });

        // insert all function statements
        for statement in func.body.iter() {
            use boba_script_ast::statement::Statement as S;
            match statement.deref() {
                S::Def(def) => self.insert_definition_into(inner_scope, def),
                S::Let { pattern, expr } => {
                    // TODO: implement let statement
                }
                S::Set { pattern, expr } => {
                    // TODO: implement set statement
                }
                S::Expr(expr) => {
                    // TODO: implement expr statement
                }
            }
        }

        // return the index of the new func
        Some(new_func)
    }

    pub fn insert_definition_into(&mut self, parent_scope: ScopeIndex, def: &Node<Definition>) {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // match and insert the definition item
        use boba_script_ast::Definition as D;
        match def.deref() {
            D::Static { vis, pattern, expr } => {
                // TODO
            }
            D::Module { vis, name, module } => {
                self.insert_module_into(parent_scope, vis, name, module);
            }
            D::Class { vis, name, class } => {
                self.insert_class_into(parent_scope, vis, name, class);
            }
            D::Func { vis, name, func } => {
                self.insert_func_into(parent_scope, vis, name, func);
            }
        }
    }

    fn build_union(union: &Node<Union>) -> TypeUnion {
        TypeUnion {
            paths: union
                .types
                .iter()
                .map(|concrete| concrete.path.iter().map(|s| s.to_string()).collect())
                .collect(),
            _private: (),
        }
    }
}
