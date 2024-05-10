use crate::{
    lexer::Token,
    parser::{Assign, Expr, Node, NodeBuilder},
    BobaError,
};

#[derive(Debug)]
pub enum ShellCommand {
    Assign(Node<Assign>),
    Expr(Node<Expr>),
}

impl ShellCommand {
    pub fn parser(builder: &mut NodeBuilder) -> Result<Self, BobaError> {
        match builder.peek() {
            Some((Token::Let, _)) => {
                let assign = builder.parse(Assign::parser)?;
                Ok(ShellCommand::Assign(assign))
            }
            _ => {
                let expr = builder.parse(Expr::parser)?;
                Ok(ShellCommand::Expr(expr))
            }
        }
    }
}
