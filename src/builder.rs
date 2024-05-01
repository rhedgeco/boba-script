use std::path::PathBuf;

use ariadne::{Label, Report, ReportKind, Span};
use logos::Logos;

use crate::{utils, Token};

pub fn build(workdir: Option<PathBuf>) {
    let input = workdir.unwrap_or_default().join("src").join("main.boba");
    let filename = input.as_os_str().to_string_lossy().to_string();
    let source = utils::read_to_source(input);
    println!("SOURCE:\n{}", source.text());

    let tokens = Token::lexer(source.text())
        .spanned()
        .filter_map(|(token, span)| match token {
            Ok(token) => Some((token, span)),
            Err(()) => {
                let token = &source.text()[span.clone()];
                Report::build(ReportKind::Error, &filename, span.start())
                    .with_code(1)
                    .with_message("Invalid token found while parsing")
                    .with_label(
                        Label::new((&filename, span.clone()))
                            .with_message(format!("Invalid token '{token}'"))
                            .with_color(ariadne::Color::Red),
                    )
                    .finish()
                    .eprint((&filename, source.clone()))
                    .unwrap();
                None
            }
        })
        .collect::<Box<[_]>>();

    println!("\nTOKENS:\n{tokens:?}")
}
