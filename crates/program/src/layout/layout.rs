use std::ops::Index;

use boba_script_ast::{
    def::Visibility, func::BodyKind, node::NodeId, path::Union, Class, Definition, Expr, Func,
    Module, Node,
};
use indexmap::IndexMap;

use crate::indexers::{ClassIndex, FuncIndex, GlobalIndex, ScopeIndex};

use super::LayoutError;

#[derive(Debug, Clone, Copy)]
pub struct Vis<T> {
    pub vis: Node<Visibility>,
    pub data: Node<T>,
}

#[derive(Debug, Clone, Copy)]
pub enum DefIndex {
    Global(GlobalIndex),
    Module(ScopeIndex),
    Class(ClassIndex),
    Func(FuncIndex),
}

#[derive(Debug, Clone)]
pub struct ScopeData {
    pub super_scope: Option<ScopeIndex>,
    pub parent_scope: Option<ScopeIndex>,
    pub defs: IndexMap<String, Vis<DefIndex>>,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub node_id: NodeId,
    pub parent_scope: ScopeIndex,
    pub inner_scope: ScopeIndex,
    pub fields: IndexMap<String, Vis<Union>>,
}

#[derive(Debug, Clone)]
pub struct FuncData {
    pub node_id: NodeId,
    pub parent_scope: ScopeIndex,
    pub inner_scope: Option<ScopeIndex>,
    pub inputs: IndexMap<String, Node<Union>>,
    pub output: Node<Union>,
}

#[derive(Debug, Clone)]
pub struct ProgramLayout {
    errors: Vec<LayoutError>,
    scopes: Vec<ScopeData>,
    globals: Vec<Node<Expr>>,
    classes: Vec<ClassData>,
    funcs: Vec<FuncData>,
}

impl Index<GlobalIndex> for ProgramLayout {
    type Output = Node<Expr>;

    fn index(&self, index: GlobalIndex) -> &Self::Output {
        &self.globals[index.raw()]
    }
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

    impl IndexMut<GlobalIndex> for ProgramLayout {
        fn index_mut(&mut self, index: GlobalIndex) -> &mut Self::Output {
            &mut self.globals[index.raw()]
        }
    }

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

    pub fn globals(&self) -> &[Node<Expr>] {
        &self.globals
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

    pub fn get_global(&self, index: GlobalIndex) -> Option<&Node<Expr>> {
        self.globals.get(index.raw())
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
        // build the initial layout with root scope
        let mut layout = Self {
            errors: Default::default(),
            globals: Default::default(),
            scopes: vec![ScopeData {
                super_scope: None,
                parent_scope: None,
                defs: Default::default(),
            }],
            classes: Default::default(),
            funcs: Default::default(),
        };

        // insert all defs into the root scope
        for def in &ast.defs {
            layout.insert_def_into(ScopeIndex::new(0), def);
        }

        // return the built layout
        layout
    }

    fn insert_global(&mut self, expr: &Node<Expr>) -> GlobalIndex {
        let global_index = GlobalIndex::new(self.globals.len());
        self.globals.push(expr.clone());
        global_index
    }

    fn insert_module(&mut self, super_scope: ScopeIndex, module: &Node<Module>) -> ScopeIndex {
        // build the module scope
        let module_scope = ScopeIndex::new(self.scopes.len());
        self.scopes.push(ScopeData {
            super_scope: Some(super_scope),
            parent_scope: None,
            defs: Default::default(),
        });

        // insert all definitions into the module
        for def in &module.defs {
            self.insert_def_into(module_scope, def);
        }

        // return the module scope
        module_scope
    }

    fn insert_class(&mut self, parent_scope: ScopeIndex, class: &Node<Class>) -> ClassIndex {
        // build the inner class scope
        let inner_scope = ScopeIndex::new(self.scopes.len());
        self.scopes.push(ScopeData {
            super_scope: self[parent_scope].super_scope,
            parent_scope: Some(parent_scope),
            defs: Default::default(),
        });

        // build all the class fields
        let mut fields = IndexMap::new();
        for field in class.fields.iter() {
            fields.insert(
                field.name.to_string(),
                Vis {
                    vis: field.vis.clone(),
                    data: field.union.clone(),
                },
            );
        }

        // build the class data
        let class_index = ClassIndex::new(self.classes.len());
        self.classes.push(ClassData {
            node_id: class.id,
            parent_scope,
            inner_scope,
            fields,
        });

        // insert all definitions into the inner class scope
        for def in class.defs.iter() {
            self.insert_def_into(inner_scope, def);
        }

        // return the class index
        class_index
    }

    fn insert_func(&mut self, parent_scope: ScopeIndex, func: &Node<Func>) -> FuncIndex {
        // build the func output and inputs
        let output = func.output.clone();
        let mut inputs = IndexMap::new();
        for field in func.inputs.iter() {
            let union = field.union.clone();
            inputs.insert(field.name.to_string(), union);
        }

        // create the func data with no body
        let func_index = FuncIndex::new(self.funcs.len());
        self.funcs.push(FuncData {
            node_id: func.id,
            parent_scope,
            inputs,
            output,
            inner_scope: None,
        });

        // populate the function body if necessary
        if let BodyKind::Script(statements) = &func.body {
            // create the functions inner scope
            let inner_scope = ScopeIndex::new(self.scopes.len());
            self[func_index].inner_scope = Some(inner_scope);
            self.scopes.push(ScopeData {
                super_scope: self[parent_scope].super_scope,
                parent_scope: Some(parent_scope),
                defs: Default::default(),
            });

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

        // return the func index
        func_index
    }

    fn insert_def_into(&mut self, scope: ScopeIndex, def: &Node<Definition>) {
        use boba_script_ast::def::DefKind as D;
        let def_index = match &def.kind {
            D::Global(expr) => DefIndex::Global(self.insert_global(expr)),
            D::Module(module) => DefIndex::Module(self.insert_module(scope, module)),
            D::Class(class) => DefIndex::Class(self.insert_class(scope, class)),
            D::Func(func) => DefIndex::Func(self.insert_func(scope, func)),
        };

        use indexmap::map::Entry as E;
        match self[scope].defs.entry(def.name.to_string()) {
            E::Occupied(entry) => {
                let first = entry.into_mut().data.id;
                self.errors.push(LayoutError::DuplicateIdent {
                    second: def.name.id,
                    first,
                })
            }
            E::Vacant(entry) => {
                entry.insert(Vis {
                    vis: def.vis.clone(),
                    data: Node {
                        id: def.name.id,
                        item: def_index,
                    },
                });
            }
        }
    }
}
