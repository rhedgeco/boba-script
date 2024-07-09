use crate::{
    engine::{value::ValueKind, EvalError, Value},
    Engine,
};

use super::{expr::ExprNode, node::EvalNode, Node};

pub type StatementNode<Data> = Node<Statement<Data>, Data>;

pub enum Statement<Data> {
    Expr {
        expr: ExprNode<Data>,
        closed: bool,
    },
    Assign {
        init: bool,
        lhs: ExprNode<Data>,
        rhs: ExprNode<Data>,
    },
    While {
        cond: ExprNode<Data>,
        body: Vec<StatementNode<Data>>,
    },
}

impl<Data: Clone> EvalNode<Data> for Statement<Data> {
    fn eval_node(
        node: &Node<Self, Data>,
        engine: &mut Engine<Data>,
    ) -> Result<Value, EvalError<Data>> {
        match &node.item {
            Statement::Expr { expr, closed } => {
                let value = engine.eval(expr)?;
                match closed {
                    true => Ok(Value::None),
                    false => Ok(value),
                }
            }
            Statement::Assign { init, lhs, rhs } => {
                match init {
                    false => engine.assign(lhs, rhs)?,
                    true => engine.init_assign(lhs, rhs)?,
                }

                Ok(Value::None)
            }
            Statement::While { cond, body } => loop {
                match engine.eval(cond)? {
                    Value::Bool(true) => (),
                    Value::Bool(false) => break Ok(Value::None),
                    value => {
                        break Err(EvalError::UnexpectedType {
                            expect: ValueKind::Bool,
                            found: value.kind(),
                            data: cond.data.clone(),
                        })
                    }
                }

                for statement in body {
                    engine.eval(statement)?;
                }
            },
        }
    }
}
