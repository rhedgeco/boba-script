use std::fmt::Debug;

use ariadne::{Color, Label, Report, ReportKind, Span as AriadneSpan};

use crate::cache::Span;

use super::{BinaryOpType, UnaryOpType};

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum RunError {
    UnknownVariable {
        ident: String,
        span: Span,
    },
    UnknownFunction {
        ident: String,
        span: Span,
    },
    InvalidUnary {
        op: UnaryOpType,
        vtype: String,
        span: Span,
    },
    InvalidBinary {
        op: BinaryOpType,
        vtype1: String,
        vtype2: String,
        span: Span,
    },
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },
    ParameterCount {
        expected: usize,
        found: usize,
        span: Span,
    },
    NativeCallError {
        message: String,
        span: Span,
    },
}

impl RunError {
    pub fn report(&self) -> Report<Span> {
        match self {
            RunError::UnknownVariable { ident, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_message("Unknown Variable")
                    .with_code(format!("R-001"))
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("unknown variable '{ident}'")),
                    )
            }
            RunError::UnknownFunction { ident, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_message("Unknown Function")
                    .with_code(format!("R-002"))
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("unknown function '{ident}'")),
                    )
            }
            RunError::InvalidUnary { op, vtype, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_message("Invalid Unary Operator")
                    .with_code(format!("R-003"))
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(format!("cannot use unary '{op}' prefix with '{vtype}'")),
                    )
            }
            RunError::InvalidBinary {
                op,
                vtype1,
                vtype2,
                span,
            } => Report::build(ReportKind::Error, span.source().clone(), span.start())
                .with_message("Invalid Binary Operator")
                .with_code(format!("R-004"))
                .with_label(
                    Label::new(span.clone())
                        .with_color(Color::Red)
                        .with_message(format!(
                            "'{vtype1}' does not have a valid '{op}' operator for '{vtype2}'"
                        )),
                ),

            RunError::TypeMismatch {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Error, span.source().clone(), span.start())
                .with_message("Type Mismatch")
                .with_code(format!("R-005"))
                .with_label(
                    Label::new(span.clone())
                        .with_color(Color::Red)
                        .with_message(format!("expected {expected}, found {found}")),
                ),
            RunError::ParameterCount {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Error, span.source().clone(), span.start())
                .with_message("Wrong Parameter Count")
                .with_code(format!("R-006"))
                .with_label(
                    Label::new(span.clone())
                        .with_color(Color::Red)
                        .with_message(format!(
                            "function expects {expected} parameters, found {found}"
                        )),
                ),
            RunError::NativeCallError { message, span } => {
                Report::build(ReportKind::Error, span.source().clone(), span.start())
                    .with_message("Native Call Error")
                    .with_code(format!("R-007"))
                    .with_label(
                        Label::new(span.clone())
                            .with_color(Color::Red)
                            .with_message(message),
                    )
            }
        }
        .finish()
    }
}
