use std::ops::Deref;

use boba_script_ast::{
    def::Visibility,
    node::NodeId,
    path::{ConcreteType, PathPart},
    Node,
};

use crate::{
    indexers::{FuncIndex, ScopeIndex},
    layout::DefIndex,
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

pub fn find_ident(
    layout: &ProgramLayout,
    source_scope: ScopeIndex,
    mut module_scope: ScopeIndex,
    name: impl AsRef<str>,
    id: NodeId,
) -> Result<DefIndex, ResolveError> {
    // ensure module_scope is valid for layout
    debug_assert!(
        layout.get_scope(module_scope).is_some(),
        "module_scope is invalid for this layout"
    );

    // search up the chain of parents to find the ident
    let name = name.as_ref();
    loop {
        let scope = &layout[module_scope];
        match scope.defs.get(name) {
            Some(found_def) => match found_def.vis.deref() {
                // if the definiton is private, ensure the resolved scope is the same as source
                // private definitions are only available to items within the same module
                Visibility::Private if !is_child(layout, module_scope, source_scope) => {
                    return Err(ResolveError::PrivateIdent(id));
                }
                _ => return Ok(found_def.data.item),
            },
            None => match scope.parent_scope {
                Some(parent) => module_scope = parent,
                None => return Err(ResolveError::PrivateIdent(id)),
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
                match find_ident(layout, source_scope, module_scope, name, part.id)? {
                    DefIndex::Module(scope_index) => module_scope = scope_index,
                    _ => return Err(ResolveError::NotAModule(part.id)),
                }
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
            let DefIndex::Class(class_index) =
                find_ident(layout, source_scope, module_scope, class_name, id)?
            else {
                return Err(ResolveError::NotAClass(id));
            };

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
    match find_ident(layout, source_scope, module_scope, func_name, id)? {
        DefIndex::Func(func_index) => Ok(func_index),
        _ => Err(ResolveError::NotAFunc(id)),
    }
}
