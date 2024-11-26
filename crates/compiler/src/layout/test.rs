use boba_script_ast::{def::Visibility, Class, Node};

use crate::{layout::LayoutError, ProgramLayout};

#[test]
fn insert_conflict() {
    let vis = Node::build(Visibility::Private);
    let name = Node::build("class0".to_string());
    let ast = Node::build(Class {
        fields: vec![],
        defs: vec![],
    });

    let mut layout = ProgramLayout::new();
    if let Err(errors) = layout.insert_root_class(&vis, &name, &ast) {
        panic!("failed to insert class: {errors:?}");
    }

    let Err(errors) = layout.insert_root_class(&vis, &name, &ast) else {
        panic!("duplicate class names should conflict");
    };

    assert_eq!(
        errors[0],
        LayoutError::ClassAlreadyExists {
            insert: name.id(),
            found: name.id()
        }
    )
}
