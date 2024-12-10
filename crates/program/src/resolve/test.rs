use boba_script_ast::{
    class::ClassField,
    def::{DefKind, Visibility},
    path::{ConcreteType, PathPart, Union},
    Class, Definition, Module, Node,
};

use crate::{indexers::ClassIndex, resolve::ResolvedValue, ProgramLayout};

use super::ResolvedProgram;

#[test]
fn super_mod_private_class() {
    let ast = Node::build(Module {
        defs: vec![Node::build(Definition {
            vis: Node::build(Visibility::Private),
            name: Node::build("module0".to_string()),
            kind: DefKind::Module(Node::build(Module {
                defs: vec![
                    Node::build(Definition {
                        vis: Node::build(Visibility::Private),
                        name: Node::build("class0".to_string()),
                        kind: DefKind::Class(Node::build(Class {
                            native: None,
                            fields: vec![],
                            defs: vec![],
                        })),
                    }),
                    Node::build(Definition {
                        vis: Node::build(Visibility::Private),
                        name: Node::build("module1".to_string()),
                        kind: DefKind::Module(Node::build(Module {
                            defs: vec![Node::build(Definition {
                                vis: Node::build(Visibility::Private),
                                name: Node::build("class1".to_string()),
                                kind: DefKind::Class(Node::build(Class {
                                    native: None,
                                    fields: vec![Node::build(ClassField {
                                        vis: Node::build(Visibility::Private),
                                        name: Node::build("class1field".to_string()),
                                        union: Node::build(Union {
                                            types: vec![Node::build(ConcreteType::Path(vec![
                                                Node::build(PathPart::Super),
                                                Node::build(PathPart::Super),
                                                Node::build(PathPart::Ident("module0".to_string())),
                                                Node::build(PathPart::Ident("class0".to_string())),
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
