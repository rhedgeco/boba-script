use std::{
    ops::{Deref, Index},
    slice::Iter,
};

use boba_script_ast::{
    def::Visibility, node::NodeId, Class, Definition, Func, Module, Node, Union,
};
use indexmap::IndexMap;

use crate::indexers::{ClassIndex, FuncIndex, ScopeIndex};

use super::LayoutError;

#[derive(Debug)]
pub struct DefData<T> {
    pub vis: Node<Visibility>,
    pub name_id: NodeId,
    pub data_id: NodeId,
    pub data: T,
    _private: (),
}

#[derive(Debug)]
pub struct ScopeData {
    pub super_scope: Option<ScopeIndex>,
    pub parent_scope: Option<ScopeIndex>,
    pub modules: IndexMap<String, DefData<ScopeIndex>>,
    pub classes: IndexMap<String, DefData<ClassIndex>>,
    pub funcs: IndexMap<String, DefData<FuncIndex>>,
    _private: (),
}

#[derive(Debug)]
pub struct ClassData {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub fields: IndexMap<String, DefData<TypeUnion>>,
    _private: (),
}

#[derive(Debug)]
pub struct FuncData {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub inputs: IndexMap<String, TypeUnion>,
    pub output: TypeUnion,
    _private: (),
}

#[derive(Debug)]
pub struct TypeUnion {
    pub paths: Vec<Vec<String>>,
    _private: (),
}

#[derive(Debug, Default)]
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
    pub const fn new() -> Self {
        Self {
            scopes: Vec::new(),
            classes: Vec::new(),
            funcs: Vec::new(),
        }
    }

    pub fn scopes(&self) -> Iter<ScopeData> {
        self.scopes.iter()
    }

    pub fn classes(&self) -> Iter<ClassData> {
        self.classes.iter()
    }

    pub fn funcs(&self) -> Iter<FuncData> {
        self.funcs.iter()
    }

    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    pub fn func_count(&self) -> usize {
        self.funcs.len()
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

    pub fn get_root_scope(&self) -> Option<ScopeIndex> {
        if self.scopes.is_empty() {
            return None;
        }

        Some(ScopeIndex::from_raw(0))
    }

    pub fn get_or_create_root(&mut self) -> ScopeIndex {
        if self.scopes.is_empty() {
            self.scopes.push(ScopeData {
                super_scope: None,
                parent_scope: None,
                modules: Default::default(),
                classes: Default::default(),
                funcs: Default::default(),
                _private: (),
            });
        }

        ScopeIndex::from_raw(0)
    }

    pub fn insert_root_module(
        &mut self,
        vis: &Node<Visibility>,
        name: &Node<String>,
        module: &Node<Module>,
    ) -> Result<ScopeIndex, Vec<LayoutError>> {
        let root_scope = self.get_or_create_root();
        self.insert_module_into(root_scope, vis, name, module)
    }

    pub fn insert_root_class(
        &mut self,
        vis: &Node<Visibility>,
        name: &Node<String>,
        class: &Node<Class>,
    ) -> Result<ClassIndex, Vec<LayoutError>> {
        let root_scope = self.get_or_create_root();
        self.insert_class_into(root_scope, vis, name, class)
    }

    pub fn insert_root_func(
        &mut self,
        vis: &Node<Visibility>,
        name: &Node<String>,
        func: &Node<Func>,
    ) -> Result<FuncIndex, Vec<LayoutError>> {
        let root_scope = self.get_or_create_root();
        self.insert_func_into(root_scope, vis, name, func)
    }

    #[must_use]
    pub fn insert_module_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        module: &Node<Module>,
    ) -> Result<ScopeIndex, Vec<LayoutError>> {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // try to insert the next module index into the parent scope
        use indexmap::map::Entry as E;
        let inner_scope = ScopeIndex::from_raw(self.scopes.len());
        match self[parent_scope].modules.entry(name.to_string()) {
            E::Vacant(entry) => entry.insert(DefData {
                name_id: name.id(),
                vis: vis.clone(),
                data_id: module.id(),
                data: inner_scope,
                _private: (),
            }),
            E::Occupied(entry) => {
                return Err(vec![LayoutError::ModuleAlreadyExists {
                    insert: name.id(),
                    found: entry.get().name_id,
                }])
            }
        };

        // build the scope data
        self.scopes.push(ScopeData {
            super_scope: Some(parent_scope),
            parent_scope: None, // module scopes have no parent
            modules: Default::default(),
            classes: Default::default(),
            funcs: Default::default(),
            _private: (),
        });

        // insert all definitions into the new scope
        let mut errors = Vec::new();
        for def in module.defs.iter() {
            if let Err(new_errors) = self.insert_definition_into(inner_scope, def) {
                errors.extend(new_errors)
            }
        }

        // return errors if there is any
        if !errors.is_empty() {
            return Err(errors);
        }

        // return the index of the new scope
        Ok(inner_scope)
    }

    #[must_use]
    pub fn insert_class_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        class: &Node<Class>,
    ) -> Result<ClassIndex, Vec<LayoutError>> {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // try to insert the next class index into the parent scope
        use indexmap::map::Entry as E;
        let new_class = ClassIndex::from_raw(self.classes.len());
        match self[parent_scope].classes.entry(name.to_string()) {
            E::Vacant(entry) => entry.insert(DefData {
                vis: vis.clone(),
                name_id: name.id(),
                data_id: class.id(),
                data: new_class,
                _private: (),
            }),
            E::Occupied(entry) => {
                return Err(vec![LayoutError::ClassAlreadyExists {
                    insert: name.id(),
                    found: entry.get().name_id,
                }])
            }
        };

        // build all the class fields
        let mut fields = IndexMap::new();
        for field in class.fields.iter() {
            let union = Self::build_union(&field.ty);
            fields.insert(
                field.name.to_string(),
                DefData {
                    vis: field.vis.clone(),
                    name_id: field.name.id(),
                    data_id: field.ty.id(),
                    data: union,
                    _private: (),
                },
            );
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
        let mut errors = Vec::new();
        for def in class.defs.iter() {
            if let Err(new_errors) = self.insert_definition_into(inner_scope, def) {
                errors.extend(new_errors)
            }
        }

        // return errors if there are any
        if !errors.is_empty() {
            return Err(errors);
        }

        // return the index of the new class
        Ok(new_class)
    }

    #[must_use]
    pub fn insert_func_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        func: &Node<Func>,
    ) -> Result<FuncIndex, Vec<LayoutError>> {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // try to insert the next func index into the parent scope
        use indexmap::map::Entry as E;
        let new_func = FuncIndex::from_raw(self.funcs.len());
        match self[parent_scope].funcs.entry(name.to_string()) {
            E::Vacant(entry) => entry.insert(DefData {
                vis: vis.clone(),
                name_id: name.id(),
                data_id: func.id(),
                data: new_func,
                _private: (),
            }),
            E::Occupied(entry) => {
                return Err(vec![LayoutError::FuncAlreadyExists {
                    insert: name.id(),
                    found: entry.get().name_id,
                }])
            }
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
        let mut errors = Vec::new();
        for statement in func.body.iter() {
            use boba_script_ast::statement::Statement as S;
            match statement.deref() {
                S::Def(def) => {
                    if let Err(new_errors) = self.insert_definition_into(inner_scope, def) {
                        errors.extend(new_errors)
                    }
                }
                S::Let { pattern, expr } => {
                    let _ = (pattern, expr);
                    errors.push(LayoutError::Unimplemented {
                        id: statement.id(),
                        message: "let statements are unimplemented",
                    })
                }
                S::Set { pattern, expr } => {
                    let _ = (pattern, expr);
                    errors.push(LayoutError::Unimplemented {
                        id: statement.id(),
                        message: "set statements are unimplemented",
                    })
                }
                S::Expr(expr) => {
                    let _ = expr;
                    errors.push(LayoutError::Unimplemented {
                        id: statement.id(),
                        message: "expr statements are unimplemented",
                    })
                }
            }
        }

        // return errors if there are any
        if !errors.is_empty() {
            return Err(errors);
        }

        // return the index of the new func
        Ok(new_func)
    }

    pub fn insert_definition_into(
        &mut self,
        parent_scope: ScopeIndex,
        def: &Node<Definition>,
    ) -> Result<(), Vec<LayoutError>> {
        // assert that parent scope is valid
        assert!(
            parent_scope.raw() < self.scopes.len(),
            "parent_scope is invalid for this layout"
        );

        // match and insert the definition item
        use boba_script_ast::Definition as D;
        match def.deref() {
            D::Static { vis, pattern, expr } => {
                let _ = (vis, pattern, expr);
                Err(vec![LayoutError::Unimplemented {
                    id: def.id(),
                    message: "statics are unimplemented",
                }])
            }
            D::Module { vis, name, module } => self
                .insert_module_into(parent_scope, vis, name, module)
                .map(|_| ()),
            D::Class { vis, name, class } => self
                .insert_class_into(parent_scope, vis, name, class)
                .map(|_| ()),
            D::Func { vis, name, func } => self
                .insert_func_into(parent_scope, vis, name, func)
                .map(|_| ()),
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
