use ariadne::{Color, Label, Report, ReportKind};

use crate::parser::Span;

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
    pub fn code(&self) -> usize {
        // From the docs for discriminants
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() as usize + 1 }
    }

    pub fn to_ariadne<'a>(&self, id: &'a str) -> Report<(&'a str, Span)> {
        match self {
            RunError::UnknownVariable { ident, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_message("Unknown Variable")
                    .with_code(format!("R-{:0>3}", self.code()))
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("unknown variable '{ident}'")),
                    )
            }
            RunError::UnknownFunction { ident, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_message("Unknown Function")
                    .with_code(format!("R-{:0>3}", self.code()))
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("unknown function '{ident}'")),
                    )
            }
            RunError::InvalidUnary { op, vtype, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_message("Invalid Unary Operator")
                    .with_code(format!("C-{:0>3}", self.code()))
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("cannot use unary '{op}' prefix with '{vtype}'")),
                    )
            }
            RunError::InvalidBinary {
                op,
                vtype1,
                vtype2,
                span,
            } => Report::build(ReportKind::Error, id, span.start)
                .with_message("Invalid Binary Operator")
                .with_code(format!("C-{:0>3}", self.code()))
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!(
                            "'{vtype1}' does not have a valid '{op}' operator for '{vtype2}'"
                        )),
                ),

            RunError::TypeMismatch {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Error, id, span.start)
                .with_message("Type Mismatch")
                .with_code(format!("C-{:0>3}", self.code()))
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!("expected {expected}, found {found}")),
                ),
            RunError::ParameterCount {
                expected,
                found,
                span,
            } => Report::build(ReportKind::Error, id, span.start)
                .with_message("Wrong Parameter Count")
                .with_code(format!("C-{:0>3}", self.code()))
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!(
                            "function expects {expected} parameters, found {found}"
                        )),
                ),
            RunError::NativeCallError { message, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_message("Native Call Error")
                    .with_code(format!("C-{:0>3}", self.code()))
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(message),
                    )
            }
        }
        .finish()
    }
}
