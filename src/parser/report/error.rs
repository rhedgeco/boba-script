use std::{
    num::{IntErrorKind, ParseFloatError, ParseIntError},
    ops::Range,
};

use ariadne::{Color, Label, Report, ReportKind};

use crate::lexer::token::Span;

#[derive(Debug)]
pub enum PError {
    UnexpectedEnd {
        expect: String,
        pos: usize,
    },
    UnexpectedToken {
        expect: String,
        found: String,
        span: Span,
    },
    ParseIntError {
        error: ParseIntError,
        span: Span,
    },
    ParseFloatError {
        error: ParseFloatError,
        span: Span,
    },
    UnclosedBrace {
        open_span: Span,
        close_message: String,
        close_span: Span,
    },
}

impl PError {
    pub fn as_ariadne<'a>(&self, id: &'a str) -> Report<(&'a str, Range<usize>)> {
        match self {
            PError::UnexpectedEnd { expect, pos } => Report::build(ReportKind::Error, id, *pos)
                .with_code(1)
                .with_message("Unexpected end of input")
                .with_label(
                    Label::new((id, *pos..*pos))
                        .with_color(Color::Red)
                        .with_message(format!("expected {expect}, found nothing")),
                ),
            PError::UnexpectedToken {
                expect,
                found,
                span,
            } => Report::build(ReportKind::Error, id, span.start)
                .with_code(2)
                .with_message("Unexpected token")
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!("expected {expect}, found {found}")),
                ),
            PError::ParseIntError { error, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_code(3)
                    .with_message("Failed to parse integer")
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(match error.kind() {
                                IntErrorKind::PosOverflow => {
                                    "too large. must be at max 9,223,372,036,854,775,807".into()
                                }
                                IntErrorKind::NegOverflow => {
                                    "too small. must be at min -9,223,372,036,854,775,808".into()
                                }
                                _ => format!("{error}"),
                            }),
                    )
            }
            PError::ParseFloatError { error, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_code(4)
                    .with_message("Failed to parse integer")
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(error),
                    )
            }
            PError::UnclosedBrace {
                open_span,
                close_message,
                close_span,
            } => Report::build(ReportKind::Error, id, open_span.start)
                .with_code(5)
                .with_message("Unclosed brace")
                .with_label(
                    Label::new((id, open_span.clone()))
                        .with_color(Color::Red)
                        .with_message("opening brace found here"),
                )
                .with_label(
                    Label::new((id, close_span.clone()))
                        .with_color(Color::Cyan)
                        .with_message(close_message),
                ),
        }
        .finish()
    }
}
