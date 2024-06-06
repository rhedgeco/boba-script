use std::fmt::Debug;

use ariadne::{Color, Label, Report, ReportKind, Span};

use crate::cache::CacheSpan;

use super::{BinaryOpType, UnaryOpType};

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum RunError<Data> {
    UnknownVariable {
        ident: String,
        data: Data,
    },
    UnknownFunction {
        ident: String,
        data: Data,
    },
    InvalidUnary {
        op: UnaryOpType,
        vtype: String,
        data: Data,
    },
    InvalidBinary {
        op: BinaryOpType,
        vtype1: String,
        vtype2: String,
        data: Data,
    },
    TypeMismatch {
        expected: String,
        found: String,
        data: Data,
    },
    ParameterCount {
        expected: usize,
        found: usize,
        data: Data,
    },
    NativeCallError {
        message: String,
        data: Data,
    },
    StringAllocError {
        data: Data,
    },
    InvalidCall {
        ident: String,
        found: String,
        data: Data,
    },
    ConstAssign {
        data: Data,
    },
}

impl RunError<CacheSpan> {
    pub fn report(&self) -> Report<CacheSpan> {
        match self {
            RunError::UnknownVariable { ident, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_message("Unknown Variable")
                    .with_code("R-001")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("unknown variable '{ident}'")),
                    )
            }
            RunError::UnknownFunction { ident, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_message("Unknown Function")
                    .with_code("R-002")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("unknown function '{ident}'")),
                    )
            }
            RunError::InvalidUnary { op, vtype, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_message("Invalid Unary Operator")
                    .with_code(format!("R-003"))
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("cannot use unary '{op}' prefix with '{vtype}'")),
                    )
            }
            RunError::InvalidBinary {
                op,
                vtype1,
                vtype2,
                data,
            } => Report::build(ReportKind::Error, data.source().clone(), data.start())
                .with_message("Invalid Binary Operator")
                .with_code(format!("R-004"))
                .with_label(
                    Label::new(data.clone())
                        .with_color(Color::Red)
                        .with_message(format!(
                            "'{vtype1}' does not have a valid '{op}' operator for '{vtype2}'"
                        )),
                ),
            RunError::TypeMismatch {
                expected,
                found,
                data,
            } => Report::build(ReportKind::Error, data.source().clone(), data.start())
                .with_message("Type Mismatch")
                .with_code("R-005")
                .with_label(
                    Label::new(data.clone())
                        .with_color(Color::Red)
                        .with_message(format!("expected {expected}, found {found}")),
                ),
            RunError::ParameterCount {
                expected,
                found,
                data,
            } => Report::build(ReportKind::Error, data.source().clone(), data.start())
                .with_message("Wrong Parameter Count")
                .with_code("R-006")
                .with_label(
                    Label::new(data.clone())
                        .with_color(Color::Red)
                        .with_message(format!(
                            "function expects {expected} parameters, found {found}"
                        )),
                ),
            RunError::NativeCallError { message, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_message("Native Call Error")
                    .with_code("R-007")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(message),
                    )
            }
            RunError::StringAllocError { data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_message("String Alloc Error")
                    .with_code("R-008")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(
                            "tried to create string longer than 9,223,372,036,854,775,807 chars",
                        ),
                    )
            }
            RunError::InvalidCall { ident, found, data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_message("Invalid Call")
                    .with_code("R-009")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message(format!("cannot call {ident}, found type {found}")),
                    )
            }
            RunError::ConstAssign { data } => {
                Report::build(ReportKind::Error, data.source().clone(), data.start())
                    .with_message("Const Assign")
                    .with_code("R-010")
                    .with_label(
                        Label::new(data.clone())
                            .with_color(Color::Red)
                            .with_message("cannot assign value to a constant"),
                    )
            }
        }
        .finish()
    }
}
