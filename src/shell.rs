use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
    iter::Peekable,
    ops::Deref,
};

use ariadne::{Color, Label, Report, ReportKind, Source, Span};
use logos::Logos;

use crate::{
    ast::{Assign, BobaError, Expr, Node, TokenIter, TokenParser},
    Token,
};

#[derive(Debug)]
enum ShellCommand {
    Assign(Node<Assign>),
    Expr(Node<Expr>),
}

impl TokenParser for ShellCommand {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, BobaError> {
        match tokens.peek() {
            Some((Token::Let, _)) => {
                let assign = Assign::parse(tokens)?;
                Ok(Node::new(assign.span().clone(), Self::Assign(assign)))
            }
            _ => {
                let expr = Expr::parse(tokens)?;
                Ok(Node::new(expr.span().clone(), Self::Expr(expr)))
            }
        }
    }
}

pub fn start_session() {
    let mut vars = HashMap::new();
    loop {
        print!("boba > ");
        stdout().flush().unwrap();

        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        let line_source = Source::from(&line);

        let tokens = Token::lexer(&line)
            .spanned()
            .filter_map(|(result, span)| match result {
                Ok(token) => Some((token, span)),
                Err(error) => {
                    Report::build(ReportKind::Error, "shell", span.start())
                        .with_code(1)
                        .with_message(format!("Tokenization Error"))
                        .with_label(
                            Label::new(("shell", span))
                                .with_color(Color::Red)
                                .with_message(error.get_message()),
                        )
                        .finish()
                        .eprint(("shell", line_source.clone()))
                        .unwrap();
                    None
                }
            })
            .collect::<Vec<_>>();

        match ShellCommand::parse(&mut tokens.into_iter().peekable()) {
            Ok(command) => match command.deref() {
                ShellCommand::Assign(assign) => match assign.expr.eval(&vars) {
                    Ok(value) => {
                        println!("{} <- {value}", assign.ident.as_str());
                        vars.insert(assign.ident.clone(), value);
                    }
                    Err(e) => e.report("shell", line_source.clone()),
                },
                ShellCommand::Expr(expr) => match expr.eval(&vars) {
                    Ok(value) => println!("{value}"),
                    Err(e) => e.report("shell", line_source.clone()),
                },
            },
            Err(e) => e.report("shell", line_source.clone()),
        }
    }
}
