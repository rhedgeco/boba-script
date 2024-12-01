use std::ops::{Deref, Index};

use boba_script_ast::{
    def::Visibility, func::BodyKind, path::Union, Class, Definition, Func, Module, Node,
};
use indexmap::IndexMap;

use crate::indexers::{ClassIndex, FuncIndex, ScopeIndex};

use super::LayoutError;

#[derive(Debug, Clone)]
pub struct VisData<T> {
    pub vis: Node<Visibility>,
    pub name: Node<String>,
    pub data: Node<T>,
}

#[derive(Debug, Clone)]
pub struct ScopeData {
    pub super_scope: Option<ScopeIndex>,
    pub parent_scope: Option<ScopeIndex>,
    pub modules: IndexMap<String, VisData<ScopeIndex>>,
    pub classes: IndexMap<String, VisData<ClassIndex>>,
    pub funcs: IndexMap<String, VisData<FuncIndex>>,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub fields: IndexMap<String, VisData<Union>>,
}

#[derive(Debug, Clone)]
pub struct FuncData {
    pub parent_scope: ScopeIndex,
    pub inner_scope: Option<ScopeIndex>,
    pub inputs: IndexMap<String, Node<Union>>,
    pub output: Node<Union>,
}

#[derive(Debug, Clone)]
pub struct ProgramLayout {
    errors: Vec<LayoutError>,
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

// impl mutable indexing privately
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
    pub fn errors(&self) -> &[LayoutError] {
        &self.errors
    }

    pub fn scopes(&self) -> &[ScopeData] {
        &self.scopes
    }

    pub fn classes(&self) -> &[ClassData] {
        &self.classes
    }

    pub fn funcs(&self) -> &[FuncData] {
        &self.funcs
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

    pub fn build(ast: &Node<Module>) -> Self {
        // build the empty layout
        let mut layout = Self {
            errors: Default::default(),
            scopes: Default::default(),
            classes: Default::default(),
            funcs: Default::default(),
        };

        // build the root scope
        let root_scope = layout.build_scope(None);

        // insert all defs into the root scope
        for def in &ast.defs {
            layout.insert_def_into(root_scope, def);
        }

        // return the built layout
        layout
    }

    fn insert_module_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        module: &Node<Module>,
    ) {
        // try to assign the new scope index to a name in the parent module
        // if the scope is a duplicate, we still build it but it doesnt get an assigned name
        use indexmap::map::Entry as E;
        let inner_scope = ScopeIndex::new(self.scopes.len());
        match self[parent_scope].modules.entry(name.to_string()) {
            E::Vacant(entry) => {
                entry.insert(VisData {
                    vis: vis.clone(),
                    name: name.clone(),
                    data: Node {
                        id: module.id,
                        item: inner_scope,
                    },
                });
            }
            E::Occupied(entry) => {
                let first = entry.get().name.id;
                self.errors.push(LayoutError::DuplicateModule {
                    first,
                    second: name.id,
                });
            }
        };

        // build the scope data
        self.scopes.push(ScopeData {
            super_scope: Some(parent_scope),
            parent_scope: None, // module scopes have no parent
            modules: Default::default(),
            classes: Default::default(),
            funcs: Default::default(),
        });

        // insert all definitions into the new scope
        for def in module.defs.iter() {
            self.insert_def_into(inner_scope, def);
        }
    }

    fn insert_class_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        class: &Node<Class>,
    ) {
        // try to assign the new class index to a name in the parent module
        // if the class is a duplicate, we still build it but it doesnt get an assigned name
        use indexmap::map::Entry as E;
        let class_index = ClassIndex::new(self.classes.len());
        match self[parent_scope].classes.entry(name.to_string()) {
            E::Vacant(entry) => {
                entry.insert(VisData {
                    vis: vis.clone(),
                    name: name.clone(),
                    data: Node {
                        id: class.id,
                        item: class_index,
                    },
                });
            }
            E::Occupied(entry) => {
                let first = entry.get().name.id;
                self.errors.push(LayoutError::DuplicateClass {
                    first,
                    second: name.id,
                });
            }
        };

        // build all the class fields
        let mut fields = IndexMap::new();
        for field in class.fields.iter() {
            fields.insert(
                field.name.to_string(),
                VisData {
                    vis: field.vis.clone(),
                    name: field.name.clone(),
                    data: field.union.clone(),
                },
            );
        }

        // build the inner scope and class data
        let inner_scope = self.build_scope(Some(parent_scope));
        self.classes.push(ClassData {
            parent_scope,
            inner_scope,
            fields,
        });

        // insert all definitions into the inner class scope
        for def in class.defs.iter() {
            self.insert_def_into(inner_scope, def);
        }
    }

    fn insert_func_into(
        &mut self,
        parent_scope: ScopeIndex,
        vis: &Node<Visibility>,
        name: &Node<String>,
        func: &Node<Func>,
    ) {
        // try to assign the new func index to a name in the parent module
        // if the func is a duplicate, we still build it but it doesnt get an assigned name
        use indexmap::map::Entry as E;
        let func_index = FuncIndex::new(self.funcs.len());
        match self[parent_scope].funcs.entry(name.to_string()) {
            E::Vacant(entry) => {
                entry.insert(VisData {
                    vis: vis.clone(),
                    name: name.clone(),
                    data: Node {
                        id: func.id,
                        item: func_index,
                    },
                });
            }
            E::Occupied(entry) => {
                let first = entry.get().name.id;
                self.errors.push(LayoutError::DuplicateFunc {
                    first,
                    second: name.id,
                });
            }
        };

        // build the func output and inputs
        let output = func.output.clone();
        let mut inputs = IndexMap::new();
        for field in func.inputs.iter() {
            let union = field.union.clone();
            inputs.insert(field.name.to_string(), union);
        }

        // create the func data with no body
        self.funcs.push(FuncData {
            parent_scope,
            inputs,
            output,
            inner_scope: None,
        });

        // populate the function body if necessary
        if let BodyKind::Script(statements) = &func.body {
            // create the functions inner scope
            let inner_scope = self.build_scope(Some(parent_scope));
            self[func_index].inner_scope = Some(inner_scope);

            // insert all function statements
            for statement in statements {
                use boba_script_ast::statement::Statement as S;
                match statement {
                    S::Global(def) => self.insert_def_into(inner_scope, def),
                    S::Local(local) => self.errors.push(LayoutError::Unimplemented {
                        id: local.id,
                        message: "local statements are currently unimplemented",
                    }),
                }
            }
        }
    }

    fn insert_def_into(&mut self, scope: ScopeIndex, def: &Node<Definition>) {
        use boba_script_ast::Definition as D;
        match def.deref() {
            D::Static { vis, pattern, expr } => {
                let _ = (vis, pattern, expr);
                self.errors.push(LayoutError::Unimplemented {
                    id: def.id,
                    message: "static variables are not currently implemented",
                });
            }
            D::Module { vis, name, module } => self.insert_module_into(scope, vis, name, module),
            D::Class { vis, name, class } => self.insert_class_into(scope, vis, name, class),
            D::Func { vis, name, func } => self.insert_func_into(scope, vis, name, func),
        }
    }

    fn build_scope(&mut self, parent_scope: Option<ScopeIndex>) -> ScopeIndex {
        let scope_index = ScopeIndex::new(self.scopes.len());
        self.scopes.push(ScopeData {
            super_scope: parent_scope.and_then(|parent_scope| self[parent_scope].super_scope),
            parent_scope,
            modules: Default::default(),
            classes: Default::default(),
            funcs: Default::default(),
        });

        scope_index
    }
}
