use boba_script_ast::{def::Visibility, Class, Node};

use crate::{layout::LayoutError, ProgramLayout};

#[test]
fn insert_conflict() {
    let vis = Node::build(Visibility::Private);
    let name1 = Node::build("class0".to_string());
    let name2 = Node::build("class0".to_string());
    let ast = Node::build(Class {
        fields: vec![],
        defs: vec![],
    });

    let mut layout = ProgramLayout::new();
    layout.insert_root_class(&vis, &name1, &ast).unwrap();
    let Err(errors) = layout.insert_root_class(&vis, &name2, &ast) else {
        panic!("duplicate class names should create a conflict");
    };

    assert_eq!(
        errors[0],
        LayoutError::ClassAlreadyExists {
            insert: name2.id(),
            found: name1.id()
        }
    )
}
