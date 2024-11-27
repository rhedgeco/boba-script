use boba_script_ast::{
    def::Visibility,
    path::{ConcreteType, Union},
    Field, Func, Node,
};
use boba_script_engine::{CallValue, Engine};
use boba_script_program::{Program, ProgramLayout};

fn main() {
    let ast = Node::build(Func {
        inputs: vec![Node::build(Field {
            name: Node::build("message".to_string()),
            union: Node::build(Union {
                types: vec![Node::build(ConcreteType::String)],
            }),
        })],
        output: Node::build(Union { types: vec![] }),
        body: vec![],
    });

    let mut layout = ProgramLayout::new();
    let func_index = layout
        .insert_root_func(
            &Node::build(Visibility::Private),
            &Node::build("print".to_string()),
            &ast,
        )
        .expect("valid layout");

    let program = Program::compile(&layout).expect("valid program");
    let mut engine = Engine::load(program);
    engine
        .function(func_index)
        .with_param(CallValue::String("Hello, World!".to_string()))
        .call()
        .expect("valid function call");
}
