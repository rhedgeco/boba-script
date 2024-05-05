use std::{
    io::{stdin, stdout, Write},
    iter::Peekable,
};

use ariadne::{Color, Label, Report, ReportKind, Source, Span};
use logos::Logos;

use crate::{
    ast::{Assign, Expr, ParserError, TokenIter, TokenParser},
    Token,
};

#[derive(Debug)]
enum ShellCommand {
    Assign(Assign),
    Expr(Expr),
}

impl TokenParser for ShellCommand {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Self::Output, ParserError> {
        match tokens.peek() {
            Some((Token::Let, _)) => Ok(Self::Assign(Assign::parse(tokens)?)),
            _ => Ok(Self::Expr(Expr::parse(tokens)?)),
        }
    }
}

pub fn start_session() {
    let mut vars = Vec::new();
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
            Ok(command) => match command {
                ShellCommand::Assign(assign) => match assign.expr.eval(&vars) {
                    None => eprintln!("Invalid expression for assignment"),
                    Some(value) => {
                        println!("{} <- {value}", assign.ident.as_str());
                        vars.push((assign.ident, value));
                    }
                },
                ShellCommand::Expr(expr) => match expr.eval(&vars) {
                    None => eprintln!("Invalid expression"),
                    Some(value) => println!("{value}"),
                },
            },
            Err(error) => {
                let mut report = Report::build(ReportKind::Error, "shell", 0)
                    .with_code(1)
                    .with_message(error.message);

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
