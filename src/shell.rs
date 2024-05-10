use std::ops::Deref;

use ariadne::{Label, Report, ReportKind, Source};
use logos::Logos;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{
    engine::Scope,
    lexer::Token,
    parser::{Assign, Expr, Node, NodeBuilder, ParseSource},
    BobaError, Engine,
};

#[derive(Debug)]
enum ShellCommand {
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

pub fn start_session() {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic(format!("boba ")),
        DefaultPromptSegment::Empty,
    );

    let engine = Engine::new();
    let mut scope = Scope::new();
    loop {
        let line = match line_editor.read_line(&prompt) {
            Ok(Signal::Success(buffer)) => Source::from(buffer),
            Ok(Signal::CtrlD) => {
                println!("Closing Shell...");
                return;
            }
            Ok(Signal::CtrlC) => {
                println!("Aborting...");
                return;
            }
            Err(e) => {
                eprintln!("Input Error: {e}");
                continue;
            }
        };

        // get all tokens for the line
        let mut tokens = Token::lexer(line.text()).spanned().map(|(result, span)| {
            (
                // panic on unexpected invalid token
                // all tokens 'should' be able to be parsed
                result.expect(&format!(
                    "unexpected invalid token '{}' while lexing",
                    &line.text()[span.clone()]
                )),
                span,
            )
        });

        // parse and evaluate tokens as a shell command
        match ParseSource::new(&mut tokens).parse(ShellCommand::parser) {
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
                        .eprint(("shell", line.clone()))
                        .unwrap(),
                },
                ShellCommand::Expr(expr) => match engine.eval(&scope, &expr) {
                    Ok(value) => println!("{value}"),
                    Err(label) => Report::build(ReportKind::Error, "shell", label.span.start)
                        .with_message("Evaluation Error")
                        .with_label(
                            Label::new(("shell", label.span))
                                .with_message(label.message)
                                .with_color(label.color),
                        )
                        .finish()
                        .eprint(("shell", line.clone()))
                        .unwrap(),
                },
            },
            Err(error) => error.report("shell", line.clone()),
        }
    }
}
