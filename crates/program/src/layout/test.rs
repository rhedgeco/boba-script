use boba_script_ast::{
    def::{DefKind, Visibility},
    func::FuncBody,
    node::NodeId,
    path::PathUnion,
    Class, Definition, Func, Module, Node,
};

use crate::{layout::LayoutError, ProgramLayout};

#[test]
fn insert_conflict() {
    const IDENT: &str = "test_ident";
    let first_class_id = NodeId::unique();
    let second_class_id = NodeId::unique();
    let ast = Node::unique(Module {
        defs: vec![
            Node::unique(Definition {
                vis: Node::unique(Visibility::Private),
                name: Node {
                    id: first_class_id,
                    item: IDENT.to_string(),
                },
                kind: DefKind::Class(Node::unique(Class {
                    native: None,
                    fields: vec![],
                    defs: vec![],
                })),
            }),
            Node::unique(Definition {
                vis: Node::unique(Visibility::Private),
                name: Node {
                    id: second_class_id,
                    item: IDENT.to_string(),
                },
                kind: DefKind::Func(Node::unique(Func {
                    parameters: vec![],
                    output: Node::unique(PathUnion { types: vec![] }),
                    body: Node::unique(FuncBody::Native),
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
