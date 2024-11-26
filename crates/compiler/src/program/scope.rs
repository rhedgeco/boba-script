use indexmap::IndexMap;

use crate::{
    indexers::{ClassIndex, FuncIndex, ScopeIndex},
    ProgramLayout,
};

#[derive(Debug, Default)]
pub struct Scope {
    classes: IndexMap<String, ClassIndex>,
    funcs: IndexMap<String, FuncIndex>,
    modules: IndexMap<String, Scope>,
}

impl Scope {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get_class(&self, name: impl AsRef<str>) -> Option<ClassIndex> {
        self.classes.get(name.as_ref()).cloned()
    }

    pub fn get_func(&self, name: impl AsRef<str>) -> Option<FuncIndex> {
        self.funcs.get(name.as_ref()).cloned()
    }

    pub fn get_module_scope(&self, name: impl AsRef<str>) -> Option<&Scope> {
        self.modules.get(name.as_ref())
    }

    pub fn compile(layout: &ProgramLayout, scope: ScopeIndex) -> Self {
        // validate scope index
        assert!(
            scope.raw() < layout.scope_count(),
            "scope is not valid for this layout"
        );
        let scope = &layout[scope];

        // load scope data
        Self {
            classes: scope
                .classes
                .iter()
                .map(|(name, def)| (name.to_string(), def.data))
                .collect(),
            funcs: scope
                .funcs
                .iter()
                .map(|(name, def)| (name.to_string(), def.data))
                .collect(),
            modules: scope
                .modules
                .iter()
                .map(|(name, def)| (name.to_string(), Scope::compile(layout, def.data)))
                .collect(),
        }
    }
}
