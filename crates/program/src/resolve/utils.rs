use std::ops::Deref;

use boba_script_ast::{
    def::Visibility,
    node::NodeId,
    path::{ConcreteType, PathPart},
    Node,
};

use crate::{
    indexers::{ClassIndex, FuncIndex, ScopeIndex},
    resolve::ResolveError,
    ProgramLayout,
};

use super::ResolvedValue;

pub fn is_child(layout: &ProgramLayout, parent: ScopeIndex, mut child: ScopeIndex) -> bool {
    // ensure child scope is valid for layout
    assert!(
        layout.get_scope(child).is_some(),
        "child scope is invalid for this layout"
    );

    // keep moving up until the parent is found
    while parent != child {
        // first check any immediate parents within the module
        child = match layout[child].parent_scope {
            Some(parent_scope) => parent_scope,
            // then check any super scope parents
            None => match layout[child].super_scope {
                Some(super_scope) => super_scope,
                // if the child has no parent or super scope
                // then we have reached the root and the relationship is false
                None => return false,
            },
        }
    }

    // if found, return true
    true
}

pub fn find_module(
    layout: &ProgramLayout,
    source_scope: ScopeIndex,
    mut module_scope: ScopeIndex,
    name: impl AsRef<str>,
    id: NodeId,
) -> Result<ScopeIndex, ResolveError> {
    // ensure module_scope is valid for layout
    assert!(
        layout.get_scope(module_scope).is_some(),
        "module_scope is invalid for this layout"
    );

    // search up the chain of parents to find the scope
    let name = name.as_ref();
    loop {
        let scope = &layout[module_scope];
        match scope.modules.get(name) {
            Some(found_scope) => match found_scope.vis.deref() {
                // if the module is private, ensure the resolved scope is the same as source
                // private modules are only available to items within the same module
                Visibility::Private if !is_child(layout, module_scope, source_scope) => {
                    return Err(ResolveError::PrivateClass(id));
                }
                _ => return Ok(*found_scope.data),
            },
            None => match scope.parent_scope {
                Some(parent) => module_scope = parent,
                None => return Err(ResolveError::ModuleNotFound(id)),
            },
        }
    }
}

pub fn find_class(
    layout: &ProgramLayout,
    source_scope: ScopeIndex,
    mut module_scope: ScopeIndex,
    name: impl AsRef<str>,
    id: NodeId,
) -> Result<ClassIndex, ResolveError> {
    // ensure module_scope is valid for layout
    assert!(
        layout.get_scope(module_scope).is_some(),
        "module_scope is invalid for this layout"
    );

    // search up the chain of parents to find the scope
    let name = name.as_ref();
    loop {
        let scope = &layout[module_scope];
        match scope.classes.get(name) {
            Some(found_class) => match found_class.vis.deref() {
                // if the class is private, ensure the resolved scope is the same as source
                // private classes are only available to items within the same module
                Visibility::Private if !is_child(layout, module_scope, source_scope) => {
                    return Err(ResolveError::PrivateClass(id));
                }
                _ => return Ok(*found_class.data),
            },
            None => match scope.parent_scope {
                Some(parent) => module_scope = parent,
                None => return Err(ResolveError::ClassNotFound(id)),
            },
        }
    }
}

pub fn find_func(
    layout: &ProgramLayout,
    source_scope: ScopeIndex,
    mut module_scope: ScopeIndex,
    name: impl AsRef<str>,
    id: NodeId,
) -> Result<FuncIndex, ResolveError> {
    // ensure module_scope is valid for layout
    assert!(
        layout.get_scope(module_scope).is_some(),
        "module_scope is invalid for this layout"
    );

    // search up the chain of parents to find the scope
    let name = name.as_ref();
    loop {
        let scope = &layout[module_scope];
        match scope.funcs.get(name) {
            Some(found_func) => match found_func.vis.deref() {
                // if the func is private, ensure the resolved scope is the same as source
                // private funcs are only available to items within the same module
                Visibility::Private if !is_child(layout, module_scope, source_scope) => {
                    return Err(ResolveError::PrivateFunc(id));
                }
                _ => return Ok(*found_func.data),
            },
            None => match scope.parent_scope {
                Some(parent) => module_scope = parent,
                None => return Err(ResolveError::FuncNotFound(id)),
            },
        }
    }
}

pub fn resolve_module<'a>(
    layout: &ProgramLayout,
    source_scope: ScopeIndex,
    module_path: impl Iterator<Item = &'a Node<PathPart>>,
) -> Result<ScopeIndex, ResolveError> {
    // ensure source_scope is valid for layout
    assert!(
        layout.get_scope(source_scope).is_some(),
        "source_scope is invalid for this layout"
    );

    // resolve the module path to a scope
    let mut module_scope = source_scope;
    for part in module_path {
        match part.deref() {
            PathPart::Super => match layout[module_scope].super_scope {
                None => return Err(ResolveError::SuperFromRoot(part.id)),
                Some(super_scope) => module_scope = super_scope,
            },
            PathPart::Ident(name) => {
                module_scope = find_module(layout, source_scope, module_scope, name, part.id)?
            }
        }
    }

    // return the resolved scope
    Ok(module_scope)
}

pub fn resolve_value<'a>(
    layout: &ProgramLayout,
    source_scope: ScopeIndex,
    class: &Node<ConcreteType>,
) -> Result<ResolvedValue, ResolveError> {
    // ensure source_scope is valid for layout
    assert!(
        layout.get_scope(source_scope).is_some(),
        "source_scope is invalid for this layout"
    );

    match class.deref() {
        ConcreteType::Any => Ok(ResolvedValue::Any),
        ConcreteType::None => Ok(ResolvedValue::None),
        ConcreteType::Bool => Ok(ResolvedValue::Bool),
        ConcreteType::Int => Ok(ResolvedValue::Int),
        ConcreteType::Float => Ok(ResolvedValue::Float),
        ConcreteType::String => Ok(ResolvedValue::String),
        ConcreteType::Path(path) => {
            // get the class name from the end of the iterator
            let mut path = path.iter();
            let (class_name, id) = match path.next_back() {
                None => return Err(ResolveError::EmptyPath),
                Some(part) => match part.deref() {
                    PathPart::Ident(name) => (name.as_str(), part.id),
                    _ => return Err(ResolveError::NotAClass(part.id)),
                },
            };

            // resolve the module and class path
            let module_scope = resolve_module(layout, source_scope, path)?;
            let class_index = find_class(layout, source_scope, module_scope, class_name, id)?;
            Ok(ResolvedValue::Class(class_index))
        }
    }
}

pub fn resolve_func<'a>(
    layout: &ProgramLayout,
    source_scope: ScopeIndex,
    mut func_path: impl DoubleEndedIterator<Item = &'a Node<PathPart>>,
) -> Result<FuncIndex, ResolveError> {
    // ensure source_scope is valid for layout
    assert!(
        layout.get_scope(source_scope).is_some(),
        "source_scope is invalid for this layout"
    );

    // get the func name from the iterator
    let (func_name, id) = match func_path.next_back() {
        None => return Err(ResolveError::EmptyPath),
        Some(part) => match part.deref() {
            PathPart::Ident(name) => (name.as_str(), part.id),
            _ => return Err(ResolveError::NotAFunc(part.id)),
        },
    };

    // resolve the module and func path
    let module_scope = resolve_module(layout, source_scope, func_path)?;
    find_func(layout, source_scope, module_scope, func_name, id)
}
