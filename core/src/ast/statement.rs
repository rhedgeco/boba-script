use crate::{
    engine::{value::ValueKind, EvalError, Value},
    Engine,
};

use super::{expr::ExprNode, node::EvalNode, Node};

pub type StatementNode<Source> = Node<Statement<Source>, Source>;

#[derive(Debug, Clone)]
pub enum Statement<Source> {
    Expr {
        expr: ExprNode<Source>,
        closed: bool,
    },
    Assign {
        init: bool,
        lhs: ExprNode<Source>,
        rhs: ExprNode<Source>,
    },
    While {
        cond: ExprNode<Source>,
        body: Vec<StatementNode<Source>>,
    },
    If {
        cond: ExprNode<Source>,
        pass: Vec<StatementNode<Source>>,
        fail: Vec<StatementNode<Source>>,
    },
}

impl<Source: Clone> EvalNode<Source> for Statement<Source> {
    fn eval_node(
        node: &Node<Self, Source>,
        engine: &mut Engine<Source>,
    ) -> Result<Value, EvalError<Source>> {
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
            Statement::While { cond, body } => {
                let mut output = Value::None;
                loop {
                    match engine.eval(cond)? {
                        Value::Bool(true) => (),
                        Value::Bool(false) => break Ok(output),
                        value => {
                            break Err(EvalError::UnexpectedType {
                                expect: ValueKind::Bool,
                                found: value.kind(),
                                source: cond.source.clone(),
                            })
                        }
                    }

                    for statement in body {
                        output = engine.eval(statement)?;
                    }
                }
            }
            Statement::If { cond, pass, fail } => {
                let mut output = Value::None;
                let statements = match engine.eval(cond)? {
                    Value::Bool(true) => pass,
                    Value::Bool(false) => fail,
                    value => {
                        return Err(EvalError::UnexpectedType {
                            expect: ValueKind::Bool,
                            found: value.kind(),
                            source: cond.source.clone(),
                        })
                    }
                };

                for statement in statements {
                    output = engine.eval(statement)?;
                }

                Ok(output)
            }
        }
    }
}
