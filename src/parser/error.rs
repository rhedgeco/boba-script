use ariadne::{Color, Label, Report, ReportKind};

use crate::parser::token::Span;

pub type PResult<'source, T> = Result<T, PError<'source>>;

#[derive(Debug)]
pub enum PError<'source> {
    InvalidToken { part: &'source str, span: Span },
    UnclosedString { quote: &'source str, span: Span },
}

impl<'source> PError<'source> {
    pub fn to_ariadne<'a>(&self, id: &'a str) -> Report<(&'a str, Span)> {
        match self {
            PError::InvalidToken { part, span } => Report::build(ReportKind::Error, id, span.start)
                .with_code("C-001")
                .with_message("Invalid Token")
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!("invalid token '{part}'")),
                )
                .finish(),
            PError::UnclosedString { quote, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_code("C-002")
                    .with_message("Unclosed String")
                    .with_labels([
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("string has no closing quote")),
                        Label::new((id, span.end..span.end))
                            .with_color(Color::Cyan)
                            .with_message(format!("expected {quote} at or before here")),
                    ])
                    .finish()
            }
        }
    }
}
