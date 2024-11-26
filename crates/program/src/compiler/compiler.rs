use std::ops::Deref;

use boba_script_ast::{def::Visibility, Class, Definition, Func, Module, Node, Union};
use indexmap::IndexMap;

#[derive(Debug, Clone, Copy)]
pub(super) struct VisIndex {
    pub(super) vis: Node<Visibility>,
    pub(super) index: usize,
}

#[derive(Debug, Default)]
pub(super) struct ScopeData {
    pub(super) super_scope: Option<usize>,
    pub(super) parent_scope: Option<usize>,
    pub(super) modules: IndexMap<String, VisIndex>,
    pub(super) classes: IndexMap<String, VisIndex>,
    pub(super) funcs: IndexMap<String, VisIndex>,
}

#[derive(Debug, Default)]
pub(super) struct ClassData {
    pub(super) parent_scope: usize,
    pub(super) fields: IndexMap<String, Vec<Vec<String>>>,
    pub(super) inner_scope: usize,
}

#[derive(Debug, Default)]
pub(super) struct FuncData {
    pub(super) parent_scope: usize,
    pub(super) inputs: IndexMap<String, Vec<Vec<String>>>,
    pub(super) output: Vec<Vec<String>>,
    pub(super) inner_scope: usize,
}

#[derive(Debug, Default)]
pub struct Compiler {
    pub(super) scopes: Vec<ScopeData>,
    pub(super) classes: Vec<ClassData>,
    pub(super) funcs: Vec<FuncData>,
}

impl Compiler {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_ast(root: &Node<Module>) -> Self {
        // create compiler and initial root scope
        let mut compiler = Self::default();
        compiler.scopes.push(ScopeData::default());

        // build all the defs in the module
        for def in root.defs.iter() {
            compiler.insert_definition_into(0, def);
        }

        // return the new compiler
        compiler
    }

    fn insert_module_into(
        &mut self,
        parent_scope: usize,
        vis: &Node<Visibility>,
        name: &Node<String>,
        module: &Node<Module>,
    ) -> Option<usize> {
        // ensure scope is in bounds
        debug_assert!(
            parent_scope < self.scopes.len(),
            "parent scope is out of bounds"
        );

        // try to insert the next module index into the parent scope
        use indexmap::map::Entry as E;
        let new_scope = self.scopes.len();
        match self.scopes[parent_scope].modules.entry(name.to_string()) {
            E::Occupied(_) => return None,
            E::Vacant(entry) => entry.insert(VisIndex {
                vis: vis.clone(),
                index: new_scope,
            }),
        };

        // get the super scope of the parent
        let super_scope = self.scopes[parent_scope].super_scope;

        // build the scope data
        self.scopes.push(ScopeData {
            super_scope,
            ..Default::default()
        });

        // insert all definitions into the new scope
        for def in module.defs.iter() {
            self.insert_definition_into(new_scope, def);
        }

        // return the index of the new scope
        Some(new_scope)
    }

    fn insert_class_into(
        &mut self,
        parent_scope: usize,
        vis: &Node<Visibility>,
        name: &Node<String>,
        class: &Node<Class>,
    ) -> Option<usize> {
        // ensure scope is in bounds
        debug_assert!(
            parent_scope < self.scopes.len(),
            "parent scope is out of bounds"
        );

        // try to insert the next class index into the parent scope
        use indexmap::map::Entry as E;
        let new_class = self.classes.len();
        match self.scopes[parent_scope].classes.entry(name.to_string()) {
            E::Occupied(_) => return None,
            E::Vacant(entry) => entry.insert(VisIndex {
                vis: vis.clone(),
                index: new_class,
            }),
        };

        // build all the class fields
        let mut fields = IndexMap::new();
        for field in class.fields.iter() {
            let ty = Self::build_union(&field.ty);
            fields.insert(field.name.to_string(), ty);
        }

        // get the super scope of the parent
        let super_scope = self.scopes[parent_scope].super_scope;

        // build the inner scope and class data
        let inner_scope = self.scopes.len();
        self.scopes.push(ScopeData {
            super_scope,
            parent_scope: Some(parent_scope),
            ..Default::default()
        });
        self.classes.push(ClassData {
            parent_scope,
            fields,
            inner_scope,
        });

        // insert all definitions into the new scope
        for def in class.defs.iter() {
            self.insert_definition_into(inner_scope, def);
        }

        // return the index of the new class
        Some(new_class)
    }

    fn insert_func_into(
        &mut self,
        parent_scope: usize,
        vis: &Node<Visibility>,
        name: &Node<String>,
        func: &Node<Func>,
    ) -> Option<usize> {
        // ensure scope is in bounds
        debug_assert!(
            parent_scope < self.scopes.len(),
            "parent scope is out of bounds"
        );

        // try to insert the next func index into the parent scope
        use indexmap::map::Entry as E;
        let new_func = self.funcs.len();
        match self.scopes[parent_scope].funcs.entry(name.to_string()) {
            E::Occupied(_) => return None,
            E::Vacant(entry) => entry.insert(VisIndex {
                vis: vis.clone(),
                index: new_func,
            }),
        };

        // build the func output and inputs
        let output = Self::build_union(&func.output);
        let mut inputs = IndexMap::new();
        for field in func.inputs.iter() {
            let ty = Self::build_union(&field.ty);
            inputs.insert(field.name.to_string(), ty);
        }

        // get the super scope of the parent
        let super_scope = self.scopes[parent_scope].super_scope;

        // build the inner scope and func data
        let inner_scope = self.scopes.len();
        self.scopes.push(ScopeData {
            super_scope,
            parent_scope: Some(parent_scope),
            ..Default::default()
        });
        self.funcs.push(FuncData {
            parent_scope,
            inputs,
            output,
            inner_scope,
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

    fn insert_definition_into(&mut self, parent_scope: usize, def: &Node<Definition>) {
        // ensure scope is in bounds
        debug_assert!(
            parent_scope < self.scopes.len(),
            "parent scope is out of bounds"
        );

        // match and insert definition
        use boba_script_ast::Definition as D;
        match def.deref() {
            D::Static { vis, pattern, expr } => {
                // TODO: implement static
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

    fn build_union(union: &Node<Union>) -> Vec<Vec<String>> {
        union
            .types
            .iter()
            .map(|concrete| concrete.path.iter().map(|s| s.to_string()).collect())
            .collect()
    }
}
