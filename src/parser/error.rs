use std::{
    fmt::Debug,
    num::{IntErrorKind, ParseFloatError, ParseIntError},
};

use ariadne::{Color, Label, Report, ReportKind, Span};

use crate::cache::CacheSpan;

pub type PResult<T, Data> = Result<T, PError<Data>>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum PError<Data> {
    UnexpectedEnd {
        expected: String,
        data: Data,
    },
    InvalidToken {
        part: String,
        data: Data,
    },
    UnclosedString {
        data: Data,
    },
    ParseIntError {
        error: ParseIntError,
        data: Data,
    },
    ParseFloatError {
        error: ParseFloatError,
        data: Data,
    },
    UnexpectedToken {
        expected: String,
        found: String,
        data: Data,
    },
    UnclosedBrace {
        data: Data,
    },
    InvalidWalrusAssignment {
        data: Data,
    },
    MixedTabsAndSpaces {
        data: Data,
        tab: bool,
    },
}

impl PError<CacheSpan> {
    pub fn report(&self) -> Report<CacheSpan> {
        match self {
            PError::UnexpectedEnd { expected, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-001"))
                    .with_message("Unexpected End of Input")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("expected {expected}, found end of input")),
                    )
                    .finish()
            }
            PError::InvalidToken { part, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-002"))
                    .with_message("Invalid Token")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("invalid token {part}")),
                    )
                    .finish()
            }
            PError::UnclosedString { data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-003"))
                    .with_message("Unclosed String")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("string has no closing quote")),
                    )
                    .finish()
            }
            PError::ParseIntError { error, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-004"))
                    .with_message("Invalid Integer")
                    .with_label(
                        Label::new(data.clone())
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
            PError::ParseFloatError { error, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-005"))
                    .with_message("Invalid Integer")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("{error}")),
                    )
                    .finish()
            }
            PError::UnexpectedToken {
                expected,
                found,
                data,
            } => Report::build(ReportKind::Error, data.source().clone(), data.start())
                .with_code(format!("C-006"))
                .with_message("Unexpected Token")
                .with_label(
                    Label::new(data.clone())
                        .with_color(Color::Red)
                        .with_message(format!("expected {expected}, found {found}")),
                )
                .finish(),
            PError::UnclosedBrace { data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-007"))
                    .with_message("Unclosed Brace")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("opening brace has no closing brace")),
                    )
                    .finish()
            }
            PError::InvalidWalrusAssignment { data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-008"))
                    .with_message("Invalid Walrus Assignment")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!(
                                "cannot assign expression to another expression"
                            )),
                    )
                    .finish()
            }
            PError::MixedTabsAndSpaces { data, tab } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_code(format!("C-009"))
                    .with_message("Mixed Tabs and Spaces")
                    .with_label(
                        Label::new(data.clone())
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
