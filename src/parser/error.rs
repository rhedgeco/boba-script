use std::{
    fmt::Debug,
    num::{IntErrorKind, ParseFloatError, ParseIntError},
};

use ariadne::{Color, Label, Report, ReportKind, Span as AriadneSpan};

use crate::cache::Span;

pub type PResult<T> = Result<T, PError>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum PError {
    UnexpectedEndOfLine {
        expected: String,
        span: Span,
    },
    InvalidToken {
        part: String,
        span: Span,
    },
    UnclosedString {
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
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    UnclosedBrace {
        span: Span,
    },
    InvalidWalrusAssignment {
        span: Span,
    },
    MixedTabsAndSpaces {
        span: Span,
        tab: bool,
    },
}

impl PError {
    pub fn report(&self) -> Report<Span> {
        match self {
            PError::UnexpectedEndOfLine { expected, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-001"))
                    .with_message("Unexpected Line End")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("expected {expected}, found end of line")),
                    )
                    .finish()
            }
            PError::InvalidToken { part, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-002"))
                    .with_message("Invalid Token")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("invalid token {part}")),
                    )
                    .finish()
            }
            PError::UnclosedString { span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-003"))
                    .with_message("Unclosed String")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("string has no closing quote")),
                    )
                    .finish()
            }
            PError::ParseIntError { error, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-004"))
                    .with_message("Invalid Integer")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(match error.kind() {
                                IntErrorKind::PosOverflow => {
                                    format!("too large. must be at max 9,223,372,036,854,775,807")
                                }
                                IntErrorKind::NegOverflow => {
                                    format!("too small. must be at min -9,223,372,036,854,775,808")
                                }
                                _ => format!("{error}"),
                            }),
                    )
                    .finish()
            }
            PError::ParseFloatError { error, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-005"))
                    .with_message("Invalid Integer")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("{error}")),
                    )
                    .finish()
            }
            PError::UnexpectedToken {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Error, span.source().clone(), span.start())
                .with_code(format!("C-006"))
                .with_message("Unexpected Token")
                .with_label(
                    Label::new(span.clone())
                        .with_color(Color::Red)
                        .with_message(format!("expected {expected}, found {found}")),
                )
                .finish(),
            PError::UnclosedBrace { span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-007"))
                    .with_message("Unclosed Brace")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("opening brace has no closing brace")),
                    )
                    .finish()
            }
            PError::InvalidWalrusAssignment { span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-008"))
                    .with_message("Invalid Walrus Assignment")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!(
                                "cannot assign expression to another expression"
                            )),
                    )
                    .finish()
            }
            PError::MixedTabsAndSpaces { span, tab } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_code(format!("C-009"))
                    .with_message("Mixed Tabs and Spaces")
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(match tab {
                                true => format!("tab found here when a space was expected"),
                                false => format!("space found here when a tab was expected"),
                            }),
                    )
                    .finish()
            }
        }
    }
}
