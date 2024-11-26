use crate::{
    indexers::{ClassIndex, FuncIndex, ScopeIndex},
    CompileError, ProgramLayout,
};

pub fn find_module(
    layout: &ProgramLayout,
    from_scope: ScopeIndex,
    name: impl AsRef<str>,
) -> Option<ScopeIndex> {
    // ensure from_scope is valid for layout
    assert!(
        layout.get_scope(from_scope).is_some(),
        "from_scope is invalid for this layout"
    );

    // search up the chain of parents to find the scope
    let name = name.as_ref();
    let mut resolve_scope = from_scope;
    loop {
        let scope = &layout[resolve_scope];
        match scope.modules.get(name) {
            Some(found_scope) => return Some(*found_scope),
            None => match scope.parent_scope {
                Some(parent) => resolve_scope = parent,
                None => return None,
            },
        }
    }
}

pub fn find_class(
    layout: &ProgramLayout,
    from_scope: ScopeIndex,
    name: impl AsRef<str>,
) -> Option<ClassIndex> {
    // ensure from_scope is valid for layout
    assert!(
        layout.get_scope(from_scope).is_some(),
        "from_scope is invalid for this layout"
    );

    // search up the chain of parents to find the scope
    let name = name.as_ref();
    let mut resolve_scope = from_scope;
    loop {
        let scope = &layout[resolve_scope];
        match scope.classes.get(name) {
            Some(found_class) => return Some(*found_class),
            None => match scope.parent_scope {
                Some(parent) => resolve_scope = parent,
                None => return None,
            },
        }
    }
}

pub fn find_func(
    layout: &ProgramLayout,
    from_scope: ScopeIndex,
    name: impl AsRef<str>,
) -> Option<FuncIndex> {
    // ensure from_scope is valid for layout
    assert!(
        layout.get_scope(from_scope).is_some(),
        "from_scope is invalid for this layout"
    );

    // search up the chain of parents to find the scope
    let name = name.as_ref();
    let mut resolve_scope = from_scope;
    loop {
        let scope = &layout[resolve_scope];
        match scope.funcs.get(name) {
            Some(found_class) => return Some(*found_class),
            None => match scope.parent_scope {
                Some(parent) => resolve_scope = parent,
                None => return None,
            },
        }
    }
}

pub fn resolve_module<S: AsRef<str>>(
    layout: &ProgramLayout,
    from_scope: ScopeIndex,
    module_path: impl Iterator<Item = S>,
) -> Result<ScopeIndex, CompileError> {
    // ensure from_scope is valid for layout
    assert!(
        layout.get_scope(from_scope).is_some(),
        "from_scope is invalid for this layout"
    );

    // resolve the module path to a scope
    let mut resolve_scope = from_scope;
    for part in module_path {
        match part.as_ref() {
            "super" => match layout[resolve_scope].super_scope {
                None => return Err(CompileError::SuperFromRootScope),
                Some(super_scope) => resolve_scope = super_scope,
            },
            part => match find_module(layout, resolve_scope, part) {
                None => return Err(CompileError::ModuleDoesNotExist),
                Some(module_scope) => resolve_scope = module_scope,
            },
        }
    }

    // return the resolved scope
    Ok(resolve_scope)
}

pub fn resolve_class<S: AsRef<str>>(
    layout: &ProgramLayout,
    from_scope: ScopeIndex,
    mut class_path: impl DoubleEndedIterator<Item = S>,
) -> Result<ClassIndex, CompileError> {
    // ensure from_scope is valid for layout
    assert!(
        layout.get_scope(from_scope).is_some(),
        "from_scope is invalid for this layout"
    );

    // get the class name from the iterator
    let class_name = match class_path.next_back() {
        None => return Err(CompileError::EmptyPath),
        Some(name) => name,
    };

    // resolve the module and class path
    let module_scope = resolve_module(layout, from_scope, class_path)?;
    match find_class(layout, module_scope, class_name) {
        None => Err(CompileError::ClassDoesNotExist),
        Some(class_index) => Ok(class_index),
    }
}

pub fn resolve_func<S: AsRef<str>>(
    layout: &ProgramLayout,
    from_scope: ScopeIndex,
    mut func_path: impl DoubleEndedIterator<Item = S>,
) -> Result<FuncIndex, CompileError> {
    // ensure from_scope is valid for layout
    assert!(
        layout.get_scope(from_scope).is_some(),
        "from_scope is invalid for this layout"
    );

    // get the func name from the iterator
    let func_name = match func_path.next_back() {
        None => return Err(CompileError::EmptyPath),
        Some(name) => name,
    };

    // resolve the module and func path
    let module_scope = resolve_module(layout, from_scope, func_path)?;
    match find_func(layout, module_scope, func_name) {
        None => Err(CompileError::ClassDoesNotExist),
        Some(func_index) => Ok(func_index),
    }
}
