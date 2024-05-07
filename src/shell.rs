use std::{
    io::{stdin, stdout, Write},
    iter::Peekable,
    ops::Deref,
};

use ariadne::{Color, Label, Report, ReportKind, Source, Span};
use logos::Logos;

use crate::{
    ast::{Assign, Expr, Node, TokenIter, TokenParser},
    engine::Scope,
    Engine, LangError, Token,
};

#[derive(Debug)]
enum ShellCommand {
    Assign(Node<Assign>),
    Expr(Node<Expr>),
}

impl TokenParser for ShellCommand {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, LangError> {
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
    let engine = Engine::new();
    let mut scope = Scope::new();
    loop {
        // print line prefix
        print!("boba > ");
        stdout().flush().unwrap();

        // wait for user input
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        let line_source = Source::from(&line);

        // convert characters into tokens
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

        // parse and evaluate shell command
        match ShellCommand::parse(&mut tokens.into_iter().peekable()) {
            Ok(command) => match command.deref() {
                ShellCommand::Assign(assign) => match engine.eval(&scope, &assign.expr) {
                    Ok(value) => {
                        println!("{} = {value}", assign.ident);
                        scope.init_var(assign.ident.clone(), value);
                    }
                    Err(label) => Report::build(ReportKind::Error, "shell", label.span.start)
                        .with_message("Evaluation Error")
                        .with_label(
                            Label::new(("shell", label.span))
                                .with_message(label.message)
                                .with_color(label.color),
                        )
                        .finish()
                        .eprint(("shell", line_source.clone()))
                        .unwrap(),
                },
                ShellCommand::Expr(expr) => match engine.eval(&scope, expr) {
                    Ok(value) => println!("{value}"),
                    Err(label) => Report::build(ReportKind::Error, "shell", label.span.start)
                        .with_message("Evaluation Error")
                        .with_label(
                            Label::new(("shell", label.span))
                                .with_message(label.message)
                                .with_color(label.color),
                        )
                        .finish()
                        .eprint(("shell", line_source.clone()))
                        .unwrap(),
                },
            },
            Err(error) => {
                let mut report =
                    Report::build(ReportKind::Error, "shell", 0).with_message(error.message);

                for label in error.labels {
                    report.add_label(
                        Label::new(("shell", label.span))
                            .with_color(label.color)
                            .with_message(label.message),
                    )
                }

                report
                    .finish()
                    .eprint(("shell", line_source.clone()))
                    .unwrap();
            }
        }
    }
}
