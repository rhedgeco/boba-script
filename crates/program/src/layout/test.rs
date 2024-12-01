use boba_script_ast::{
    def::{DefKind, Visibility},
    node::NodeId,
    Class, Definition, Module, Node,
};

use crate::{layout::LayoutError, ProgramLayout};

#[test]
fn insert_conflict() {
    const CLASS_NAME: &str = "TestClass";
    let first_class_id = NodeId::new();
    let second_class_id = NodeId::new();
    let ast = Node::build(Module {
        defs: vec![
            Node::build(Definition {
                vis: Node::build(Visibility::Private),
                name: Node {
                    id: first_class_id,
                    item: CLASS_NAME.to_string(),
                },
                kind: DefKind::Class(Node::build(Class {
                    fields: vec![],
                    defs: vec![],
                })),
            }),
            Node::build(Definition {
                vis: Node::build(Visibility::Private),
                name: Node {
                    id: second_class_id,
                    item: CLASS_NAME.to_string(),
                },
                kind: DefKind::Class(Node::build(Class {
                    fields: vec![],
                    defs: vec![],
                })),
            }),
        ],
    });

    let layout = ProgramLayout::build(&ast);
    assert_eq!(layout.errors().len(), 1);
    assert_eq!(
        layout.errors()[0],
        LayoutError::DuplicateClass {
            first: first_class_id,
            second: second_class_id
        }
    )
}
