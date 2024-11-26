use boba_script_ast::{
    class::ClassField, def::Visibility, union::ConcreteType, Class, Definition, Module, Node, Union,
};

use crate::{indexers::ClassIndex, ProgramLayout};

use super::Program;

#[test]
fn super_mod_private_class() {
    let ast = Node::build(Module {
        defs: vec![
            Node::build(Definition::Class {
                vis: Node::build(Visibility::Private),
                name: Node::build("class0".to_string()),
                class: Node::build(Class {
                    fields: vec![],
                    defs: vec![],
                }),
            }),
            Node::build(Definition::Module {
                vis: Node::build(Visibility::Private),
                name: Node::build("module1".to_string()),
                module: Node::build(Module {
                    defs: vec![Node::build(Definition::Class {
                        vis: Node::build(Visibility::Private),
                        name: Node::build("class1".to_string()),
                        class: Node::build(Class {
                            fields: vec![Node::build(ClassField {
                                vis: Node::build(Visibility::Private),
                                name: Node::build("class1field".to_string()),
                                ty: Node::build(Union {
                                    types: vec![Node::build(ConcreteType {
                                        path: vec![
                                            Node::build("super".to_string()),
                                            Node::build("super".to_string()),
                                            Node::build("module0".to_string()),
                                            Node::build("class0".to_string()),
                                        ],
                                    })],
                                }),
                            })],
                            defs: vec![],
                        }),
                    })],
                }),
            }),
        ],
    });

    let mut layout = ProgramLayout::new();
    layout
        .insert_root_module(
            &Node::build(Visibility::Private),
            &Node::build("module0".to_string()),
            &ast,
        )
        .expect("valid module");

    match Program::compile(&layout) {
        Err(errors) => panic!("failed to compile program: {errors:?}"),
        Ok(program) => {
            let class2 = program
                .get_class(ClassIndex::from_raw(1))
                .expect("valid class");

            let field = class2.get_field("class1field").expect("valid field");
            assert_eq!(field, &[ClassIndex::from_raw(0)]);
        }
    }
}
