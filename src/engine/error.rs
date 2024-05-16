use ariadne::{Color, Label, Report, ReportKind};

use crate::{ast::Ident, token::Span};

use super::{BinaryOpType, UnaryOpType};

pub enum RunError {
    UnknownVariable {
        ident: Ident,
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
}

impl RunError {
    pub fn as_ariadne<'a>(&self, id: &'a str) -> Report<(&'a str, Span)> {
        match self {
            RunError::UnknownVariable { ident, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_message("Unknown Variable")
                    .with_code("R-001")
                    .with_label(
                        Label::new((id, span.clone()))
                            .with_color(Color::Red)
                            .with_message(format!("unknown variable '{ident}'")),
                    )
            }
            RunError::InvalidUnary { op, vtype, span } => {
                Report::build(ReportKind::Error, id, span.start)
                    .with_message("Invalid Unary Operator")
                    .with_code("R-002")
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
                .with_code("R-003")
                .with_label(
                    Label::new((id, span.clone()))
                        .with_color(Color::Red)
                        .with_message(format!(
                            "'{vtype1}' does not have a valid '{op}' operator for '{vtype2}'"
                        )),
                ),
        }
        .finish()
    }
}
