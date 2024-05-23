use std::num::{IntErrorKind, ParseFloatError, ParseIntError};

use ariadne::{Color, Label, Report, ReportKind};

use crate::parser::Span;

pub type PResult<T> = Result<T, PError>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum PError {
    EndOfLine {
        expected: String,
        pos: usize,
    },
    InvalidToken {
        part: String,
        span: Span,
    },
    UnclosedString {
        quote: String,
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
        found: String,
        open: Span,
        close: Span,
    },
    InvalidWalrusAssignment {
        walrus_span: Span,
        expr_span: Span,
    },
}

impl PError {
    pub fn code(&self) -> usize {
        // From the docs for discriminants
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() as usize + 1 }
    }

    pub fn to_ariadne<'a>(&self, id: &'a str) -> Report<(&'a str, Span)> {
        match self {
            PError::EndOfLine { expected, pos } => Report::build(ReportKind::Error, id, *pos)
                .with_code(format!("C-{:0>3}", self.code()))
                .with_message("Unexpected Line End")
                .with_label(
                    Label::new((id, *pos..*pos))
                        .with_color(Color::Red)
                        .with_message(format!("expected '{expected}', found end of line")),
                )
                .finish(),
            PError::InvalidToken { part, span } => Report::build(ReportKind::Error, id, span.start)
                .with_code(format!("C-{:0>3}", self.code()))
                .with_message("Invalid Token")
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!("invalid token '{part}'")),
                )
                .finish(),
            PError::UnclosedString { quote, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_code(format!("C-{:0>3}", self.code()))
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
            PError::ParseIntError { error, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_code(format!("C-{:0>3}", self.code()))
                    .with_message("Invalid Integer")
                    .with_label(
                        Label::new((id, span.clone()))
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
                Report::build(ReportKind::Error, id, span.start)
                    .with_code(format!("C-{:0>3}", self.code()))
                    .with_message("Invalid Integer")
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("{error}")),
                    )
                    .finish()
            }
            PError::UnexpectedToken {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Error, id, span.start)
                .with_code(format!("C-{:0>3}", self.code()))
                .with_message("Unexpected Token")
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!("expected {expected}, found {found}")),
                )
                .finish(),
            PError::UnclosedBrace { found, open, close } => {
                Report::build(ReportKind::Error, id, open.start)
                    .with_code(format!("C-{:0>3}", self.code()))
                    .with_message("Unclosed Brace")
                    .with_labels([
                        Label::new((id, open.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("opening brace has no closing brace")),
                        Label::new((id, close.clone()))
                            .with_color(Color::Cyan)
                            .with_message(format!("expected closing brace here, found {found}")),
                    ])
                    .finish()
            }
            PError::InvalidWalrusAssignment {
                walrus_span,
                expr_span,
            } => Report::build(ReportKind::Error, id, expr_span.start)
                .with_code(format!("C-{:0>3}", self.code()))
                .with_message("Invalid Walrus Assignment")
                .with_labels([
                    Label::new((id, walrus_span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!("cannot assign expression to another expression")),
                    Label::new((id, expr_span.clone()))
                        .with_color(Color::Cyan)
                        .with_message(format!("expected variable, found expression")),
                ])
                .finish(),
        }
    }
}
