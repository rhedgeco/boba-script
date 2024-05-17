use std::num::{IntErrorKind, ParseFloatError, ParseIntError};

use ariadne::{Color, Label, Report, ReportKind};

use crate::token::Span;

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
    InvalidIdent {
        ident: String,
        span: Span,
    },
    IncompleteTernary {
        if_span: Span,
        end: usize,
    },
    AssignmentError {
        lhs_span: Span,
        assign_span: Span,
    },
}

impl PError {
    pub fn as_ariadne<'a>(&self, id: &'a str) -> Report<(&'a str, Span)> {
        match self {
            PError::UnexpectedEnd { expect, pos } => Report::build(ReportKind::Error, id, *pos)
                .with_code("C-001")
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
                .with_code("C-002")
                .with_message("Unexpected token")
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!("expected {expect}, found {found}")),
                ),
            PError::ParseIntError { error, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_code("C-003")
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
                    .with_code("C-004")
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
                .with_code("C-005")
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
            PError::InvalidIdent { ident, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_code("C-006")
                    .with_message("Invalid identifier")
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("'{ident}' is an invalid identifier")),
                    )
            }
            PError::IncompleteTernary { if_span, end } => {
                Report::build(ReportKind::Error, id, if_span.start)
                    .with_code("C-007")
                    .with_message("Incomplete Ternary")
                    .with_label(
                        Label::new((id, if_span.clone()))
                            .with_color(Color::Red)
                            .with_message("if condition found here"),
                    )
                    .with_label(
                        Label::new((id, *end..*end))
                            .with_color(Color::Cyan)
                            .with_message(format!(
                                "expected 'else' after expression, but found nothing"
                            )),
                    )
            }
            PError::AssignmentError {
                lhs_span,
                assign_span,
            } => Report::build(ReportKind::Error, id, lhs_span.start)
                .with_code("C-008")
                .with_message("Assignment Error")
                .with_label(
                    Label::new((id, lhs_span.clone()))
                        .with_color(Color::Red)
                        .with_message("trying to assign value to expression"),
                )
                .with_label(
                    Label::new((id, assign_span.clone()))
                        .with_color(Color::Cyan)
                        .with_message("assignment happens here"),
                )
                .with_help("did you mean to use '=='?"),
        }
        .finish()
    }
}
