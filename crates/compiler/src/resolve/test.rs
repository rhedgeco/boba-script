use boba_script_ast::{
    class::Field,
    def::{DefKind, Visibility},
    path::{PathPart, PathUnion, TypePath},
    Class, Definition, Module, Node,
};

use crate::{indexers::ClassIndex, resolve::data::ResolvedValue, ProgramLayout};

use super::ResolvedProgram;

#[test]
fn super_mod_private_class() {
    let ast = Node::unique(Module {
        defs: vec![Node::unique(Definition {
            vis: Node::unique(Visibility::Private),
            name: Node::unique("module0".to_string()),
            kind: DefKind::Module(Node::unique(Module {
                defs: vec![
                    Node::unique(Definition {
                        vis: Node::unique(Visibility::Private),
                        name: Node::unique("class0".to_string()),
                        kind: DefKind::Class(Node::unique(Class {
                            native: None,
                            fields: vec![],
                            defs: vec![],
                        })),
                    }),
                    Node::unique(Definition {
                        vis: Node::unique(Visibility::Private),
                        name: Node::unique("module1".to_string()),
                        kind: DefKind::Module(Node::unique(Module {
                            defs: vec![Node::unique(Definition {
                                vis: Node::unique(Visibility::Private),
                                name: Node::unique("class1".to_string()),
                                kind: DefKind::Class(Node::unique(Class {
                                    native: None,
                                    fields: vec![Node::unique(Field {
                                        vis: Node::unique(Visibility::Private),
                                        name: Node::unique("class1field".to_string()),
                                        union: Node::unique(PathUnion {
                                            types: vec![Node::unique(TypePath::Path(vec![
                                                Node::unique(PathPart::Super),
                                                Node::unique(PathPart::Super),
                                                Node::unique(PathPart::Ident(
                                                    "module0".to_string(),
                                                )),
                                                Node::unique(PathPart::Ident("class0".to_string())),
                                            ]))],
                                        }),
                                    })],
                                    defs: vec![],
                                })),
                            })],
                        })),
                    }),
                ],
            })),
        })],
    });

    let layout = ProgramLayout::build(&ast);
    let resolved = ResolvedProgram::resolve(&layout);
    let field = &resolved[ClassIndex::new(1)].fields[0];
    assert_eq!(field, &[ResolvedValue::Class(ClassIndex::new(0))]);
}
