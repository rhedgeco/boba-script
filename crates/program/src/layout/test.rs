use boba_script_ast::{
    def::{DefKind, Visibility},
    func::BodyKind,
    node::NodeId,
    path::Union,
    Class, Definition, Func, Module, Node,
};

use crate::{layout::LayoutError, ProgramLayout};

#[test]
fn insert_conflict() {
    const IDENT: &str = "test_ident";
    let first_class_id = NodeId::new();
    let second_class_id = NodeId::new();
    let ast = Node::build(Module {
        defs: vec![
            Node::build(Definition {
                vis: Node::build(Visibility::Private),
                name: Node {
                    id: first_class_id,
                    item: IDENT.to_string(),
                },
                kind: DefKind::Class(Node::build(Class {
                    native: None,
                    fields: vec![],
                    defs: vec![],
                })),
            }),
            Node::build(Definition {
                vis: Node::build(Visibility::Private),
                name: Node {
                    id: second_class_id,
                    item: IDENT.to_string(),
                },
                kind: DefKind::Func(Node::build(Func {
                    inputs: vec![],
                    output: Node::build(Union { types: vec![] }),
                    body: BodyKind::Native,
                })),
            }),
        ],
    });

    let layout = ProgramLayout::build(&ast);
    assert_eq!(layout.errors().len(), 1);
    assert_eq!(
        layout.errors()[0],
        LayoutError::DuplicateIdent {
            first: first_class_id,
            second: second_class_id
        }
    )
}
