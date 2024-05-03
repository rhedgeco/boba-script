use std::path::Path;

use ariadne::{Label, Report, ReportKind, Span};

use crate::{token::TokenLine, utils};

pub fn build_project(workdir: Option<impl AsRef<Path>>) {
    let input = match workdir.as_ref() {
        Some(path) => path.as_ref(),
        None => Path::new(""),
    }
    .join("src")
    .join("main.boba");

    let filename = input.as_os_str().to_string_lossy().to_string();
    let source = utils::read_to_source(input);
    println!("SOURCE:\n{}", source.text());

    println!("\nERRORS:");
    let lines = TokenLine::parse_lines(source.text())
        .filter_map(|line| match line {
            Ok(line) => Some(line),
            Err(error) => {
                Report::build(ReportKind::Error, &filename, error.span.start())
                    .with_code(1)
                    .with_message(error.message)
                    .with_label(
                        Label::new((&filename, error.span))
                            .with_message(error.label)
                            .with_color(ariadne::Color::Red),
                    )
                    .finish()
                    .eprint((&filename, source.clone()))
                    .unwrap();
                None
            }
        })
        .collect::<Box<[_]>>();

    println!("\nTOKENS:");
    for line in lines.iter() {
        println!("Indent: {} - {:?}", line.indent(), line.tokens());
    }
}
