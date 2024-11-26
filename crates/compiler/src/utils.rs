use crate::{
    indexers::{ClassIndex, FuncIndex, ScopeIndex},
    CompileError, ProgramLayout,
};

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
    let mut resolved_scope = from_scope;
    for part in module_path {
        match part.as_ref() {
            "super" => match layout[resolved_scope].super_scope {
                None => return Err(CompileError::SuperFromRootScope),
                Some(super_scope) => resolved_scope = super_scope,
            },
            part => match layout[resolved_scope].modules.get(part) {
                None => return Err(CompileError::ModuleDoesNotExist),
                Some(module_scope) => resolved_scope = *module_scope,
            },
        }
    }

    // return the resolved scope
    Ok(resolved_scope)
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
    match layout[module_scope].classes.get(class_name.as_ref()) {
        None => Err(CompileError::ClassDoesNotExist),
        Some(class_index) => Ok(*class_index),
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
    match layout[module_scope].funcs.get(func_name.as_ref()) {
        None => Err(CompileError::ClassDoesNotExist),
        Some(func_index) => Ok(*func_index),
    }
}
