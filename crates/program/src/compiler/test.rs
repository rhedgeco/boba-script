use boba_script_ast::{def::Visibility, Definition, Module, Node};

use crate::Compiler;

#[test]
pub fn from_ast() {
    // build ast
    let module = Node::build(Module {
        defs: vec![
            Node::build(Definition::Module {
                vis: Node::build(Visibility::Public),
                name: Node::build("sub_module1".to_string()),
                module: Node::build(Module {
                    defs: vec![
                        Node::build(Definition::Module {
                            vis: Node::build(Visibility::Public),
                            name: Node::build("sub_module2".to_string()),
                            module: Node::build(Module { defs: vec![] }),
                        }),
                        Node::build(Definition::Module {
                            vis: Node::build(Visibility::Public),
                            name: Node::build("sub_module3".to_string()),
                            module: Node::build(Module { defs: vec![] }),
                        }),
                    ],
                }),
            }),
            Node::build(Definition::Module {
                vis: Node::build(Visibility::Public),
                name: Node::build("sub_module4".to_string()),
                module: Node::build(Module { defs: vec![] }),
            }),
        ],
    });

    // use ast to build program
    let builder = Compiler::from_ast(&module);
    assert_eq!(builder.scopes.len(), 5);
    assert_eq!(
        builder.scopes[0]
            .modules
            .get_index(0)
            .map(|(k, v)| (k.as_str(), v.index)),
        Some(("sub_module1", 1))
    );
    assert_eq!(
        builder.scopes[1]
            .modules
            .get_index(0)
            .map(|(k, v)| (k.as_str(), v.index)),
        Some(("sub_module2", 2))
    );
    assert_eq!(
        builder.scopes[1]
            .modules
            .get_index(1)
            .map(|(k, v)| (k.as_str(), v.index)),
        Some(("sub_module3", 3))
    );
    assert_eq!(
        builder.scopes[0]
            .modules
            .get_index(1)
            .map(|(k, v)| (k.as_str(), v.index)),
        Some(("sub_module4", 4))
    );
}
