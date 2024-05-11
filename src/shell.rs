use ariadne::{Label, Report, ReportKind, Source};
use logos::Logos;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{
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

pub struct Session {
    prompt: DefaultPrompt,
    line_editor: Reedline,
    engine: Engine,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("boba ")),
                DefaultPromptSegment::Empty,
            ),
            line_editor: Reedline::create(),
            engine: Engine::new(),
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_console() {
        let mut shell = Session::new();
        loop {
            let line = match shell.line_editor.read_line(&shell.prompt) {
                Ok(Signal::Success(buffer)) => buffer,
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

            // lex line into tokens
            let line = Source::from(line);
            let mut tokens = Token::lexer(line.text())
                .spanned()
                .map(|(result, span)| {
                    (
                        // panic on unexpected invalid token
                        // all tokens 'should' be able to be parsed
                        result.expect(&format!(
                            "unexpected invalid token '{}' while lexing",
                            &line.text()[span.clone()]
                        )),
                        span,
                    )
                })
                .peekable();

            // retry if there are no tokens
            if tokens.peek() == None {
                continue;
            }

            // parse line into a shell command
            let command = match ParseSource::new(&mut tokens).parse(ShellCommand::parser) {
                Ok(statement) => statement,
                Err(e) => {
                    e.report("shell", line.clone());
                    continue;
                }
            };

            // execute shell command
            match command.into_inner() {
                ShellCommand::Expr(expr) => match shell.engine.eval(&expr) {
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
                ShellCommand::Assign(assign) => match shell.engine.eval(&assign.expr) {
                    Ok(value) => {
                        println!("{} = {value}", assign.ident);
                        let ident = assign.ident.clone();
                        shell.engine.insert_var(ident, value);
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
            }
        }
    }
}
