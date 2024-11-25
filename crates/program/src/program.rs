use std::ops::Deref;

use boba_script_ast::{def::Visibility, Class, Definition, Func, Module, Node, Union};
use indexmap::IndexMap;

#[derive(Debug, Clone, Copy)]
struct VisIndex {
    vis: Node<Visibility>,
    index: usize,
}

#[derive(Debug, Default)]
struct ScopeData {
    super_scope: Option<usize>,
    parent_scope: Option<usize>,
    modules: IndexMap<String, VisIndex>,
    classes: IndexMap<String, VisIndex>,
    funcs: IndexMap<String, VisIndex>,
}

#[derive(Debug, Default)]
struct ClassData {
    parent_scope: usize,
    fields: IndexMap<String, Vec<Vec<String>>>,
    inner_scope: usize,
}

#[derive(Debug, Default)]
struct FuncData {
    parent_scope: usize,
    inputs: IndexMap<String, Vec<Vec<String>>>,
    output: Vec<Vec<String>>,
    inner_scope: usize,
}

#[derive(Debug, Default)]
pub struct ProgramBuilder {
    scopes: Vec<ScopeData>,
    classes: Vec<ClassData>,
    funcs: Vec<FuncData>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root(module: &Node<Module>) -> Self {
        // create builder and initial root scope
        let mut builder = Self::default();
        builder.scopes.push(ScopeData::default());

        // build all the defs in the module
        for def in module.defs.iter() {
            builder.insert_definition_into(0, def);
        }

        // return the new builder
        builder
    }

    pub fn insert_module(
        &mut self,
        vis: &Node<Visibility>,
        name: &Node<String>,
        module: &Node<Module>,
    ) -> Option<usize> {
        // ensure root scope is created
        if self.scopes.is_empty() {
            self.scopes.push(ScopeData::default());
        }

        // insert the module into the root scope
        self.insert_module_into(0, vis, name, module)
    }

    pub fn insert_class(
        &mut self,
        vis: &Node<Visibility>,
        name: &Node<String>,
        class: &Node<Class>,
    ) -> Option<usize> {
        // ensure root scope is created
        if self.scopes.is_empty() {
            self.scopes.push(ScopeData::default());
        }

        // insert the class into the root scope
        self.insert_class_into(0, vis, name, class)
    }

    pub fn insert_func(
        &mut self,
        vis: &Node<Visibility>,
        name: &Node<String>,
        func: &Node<Func>,
    ) -> Option<usize> {
        // ensure root scope is created
        if self.scopes.is_empty() {
            self.scopes.push(ScopeData::default());
        }

        // insert the class into the root scope
        self.insert_func_into(0, vis, name, func)
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
            let ty = Self::build_ast_union(&field.ty);
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
        let output = Self::build_ast_union(&func.output);
        let mut inputs = IndexMap::new();
        for field in func.inputs.iter() {
            let ty = Self::build_ast_union(&field.ty);
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
                    // TODO: implement statement
                }
                S::Set { pattern, expr } => {
                    // TODO: implement statement
                }
                S::Expr(expr) => {
                    // TODO: implement statement
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

    fn build_ast_union(union: &Node<Union>) -> Vec<Vec<String>> {
        union
            .types
            .iter()
            .map(|concrete| concrete.path.iter().map(|s| s.to_string()).collect())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use boba_script_ast::{Module, Node};

    use super::*;

    #[test]
    pub fn ast_module_insert() {
        // build ast
        let module = Node::build(Module {
            defs: vec![
                Node::build(Definition::Module {
                    vis: Node::build(Visibility::Public),
                    name: Node::build("sub_module1".to_string()),
                    module: Node::build(Module {
                        defs: vec![
                            Node::build(Definition::Module {
                                vis: Node::build(Visibility::Public),
                                name: Node::build("sub_module2".to_string()),
                                module: Node::build(Module { defs: vec![] }),
                            }),
                            Node::build(Definition::Module {
                                vis: Node::build(Visibility::Public),
                                name: Node::build("sub_module3".to_string()),
                                module: Node::build(Module { defs: vec![] }),
                            }),
                        ],
                    }),
                }),
                Node::build(Definition::Module {
                    vis: Node::build(Visibility::Public),
                    name: Node::build("sub_module4".to_string()),
                    module: Node::build(Module { defs: vec![] }),
                }),
            ],
        });

        // use ast to build program
        let builder = ProgramBuilder::with_root(&module);
        assert_eq!(builder.scopes.len(), 5);
        assert_eq!(
            builder.scopes[0]
                .modules
                .get_index(0)
                .map(|(k, v)| (k.as_str(), v.index)),
            Some(("sub_module1", 1))
        );
        assert_eq!(
            builder.scopes[1]
                .modules
                .get_index(0)
                .map(|(k, v)| (k.as_str(), v.index)),
            Some(("sub_module2", 2))
        );
        assert_eq!(
            builder.scopes[1]
                .modules
                .get_index(1)
                .map(|(k, v)| (k.as_str(), v.index)),
            Some(("sub_module3", 3))
        );
        assert_eq!(
            builder.scopes[0]
                .modules
                .get_index(1)
                .map(|(k, v)| (k.as_str(), v.index)),
            Some(("sub_module4", 4))
        );
    }
}
