use std::fmt::Display;

use ariadne::{Color, Label, Report, ReportKind, Span};
use boba_script::{core::engine::EvalError, parser::error::ParseError};

pub trait ToAriadne<S: Span> {
    fn to_ariadne<'a>(self) -> Report<'a, S>;
}

impl<S: Span> ToAriadne<S> for EvalError<S> {
    fn to_ariadne<'a>(self) -> Report<'a, S> {
        match self {
            EvalError::UnknownVariable { name, source } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-001")
            .with_message("Unknown Variable")
            .with_label(
                Label::new(source)
                    .with_message(format!("unknown variable {}", name))
                    .with_color(Color::Red),
            ),
            EvalError::InvalidUnaryOp { ty, op, source } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-002")
            .with_message("Invalid Unary Operator")
            .with_label(
                Label::new(source)
                    .with_message(format!("'{}' operator is not valid for '{}' types", op, ty))
                    .with_color(Color::Red),
            ),
            EvalError::InvalidBinaryOp {
                ty1,
                ty2,
                op,
                source,
            } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-003")
            .with_message("Invalid Binary Operator")
            .with_label(
                Label::new(source)
                    .with_message(format!(
                        "'{}' does not have a valid '{}' operator for '{}' types",
                        ty1, op, ty2
                    ))
                    .with_color(Color::Red),
            ),
            EvalError::InvalidAssign { source } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-004")
            .with_message("Invalid Assignment")
            .with_label(
                Label::new(source)
                    .with_message(format!("cannot assign to this expression"))
                    .with_color(Color::Red),
            ),
            EvalError::InvalidTupleSize {
                lhs_count,
                rhs_count,
                lhs_source,
                rhs_source,
            } => Report::build(
                ReportKind::Error,
                rhs_source.source().to_owned(),
                rhs_source.start(),
            )
            .with_code("R-005")
            .with_message("Invalid Tuple Destructure")
            .with_label(
                Label::new(lhs_source)
                    .with_message(format!(
                        "expected tuple with {} parameters, found {}",
                        rhs_count, lhs_count
                    ))
                    .with_color(Color::Red),
            )
            .with_label(
                Label::new(rhs_source)
                    .with_message(format!("this is a tuple with {} parameters", rhs_count))
                    .with_color(Color::Cyan),
            ),
            EvalError::InvalidTupleDestructure {
                lhs_count,
                lhs_source,
                rhs_source,
            } => Report::build(
                ReportKind::Error,
                rhs_source.source().to_owned(),
                rhs_source.start(),
            )
            .with_code("R-006")
            .with_message("Invalid Tuple Destructure")
            .with_label(
                Label::new(lhs_source)
                    .with_message(format!(
                        "cannot destructure into tuple with {} params",
                        lhs_count
                    ))
                    .with_color(Color::Red),
            )
            .with_label(
                Label::new(rhs_source)
                    .with_message(format!("this expression produces a single value"))
                    .with_color(Color::Cyan),
            ),
            EvalError::UnexpectedType {
                expect,
                found,
                source,
            } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-007")
            .with_message("Unexpected Type")
            .with_label(
                Label::new(source)
                    .with_message(format!("expected '{}', found '{}'", expect, found))
                    .with_color(Color::Red),
            ),
            EvalError::InvalidParameters {
                found,
                expect,
                source,
            } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-008")
            .with_message("Parameter Count")
            .with_label(
                Label::new(source)
                    .with_message(format!(
                        "function expects {expect} param(s). only {found} were provided"
                    ))
                    .with_color(Color::Red),
            ),
            EvalError::NativeCall { message, source } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-009")
            .with_message("Native Error")
            .with_label(
                Label::new(source)
                    .with_message(format!("{message}"))
                    .with_color(Color::Red),
            ),
            EvalError::UnknownFunction { name, source } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-010")
            .with_message("Unknown Function")
            .with_label(
                Label::new(source)
                    .with_message(format!("unknown function {name}"))
                    .with_color(Color::Red),
            ),
            EvalError::NotAFunction {
                name,
                found,
                source,
            } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("R-011")
            .with_message("Not A Function")
            .with_label(
                Label::new(source)
                    .with_message(format!(
                        "'{name}' is not a function, it is a value with type '{found}'"
                    ))
                    .with_color(Color::Red),
            ),
        }
        .finish()
    }
}

impl<S: Span, T: Display> ToAriadne<S> for ParseError<S, T> {
    fn to_ariadne<'a>(self) -> Report<'a, S> {
        match self {
            ParseError::TokenError { error, source } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("P-001")
            .with_message("Token Error")
            .with_label(
                Label::new(source)
                    .with_message(format!("{error}"))
                    .with_color(Color::Red),
            ),
            ParseError::UnexpectedInput {
                expect,
                found,
                source,
            } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("P-002")
            .with_message("Unexpected Input")
            .with_label(
                Label::new(source)
                    .with_message(match found {
                        Some(found) => format!("expected {expect}, found {found}"),
                        None => format!("expected {expect}, found end of line"),
                    })
                    .with_color(Color::Red),
            ),
            ParseError::UnclosedBrace { open, end } => {
                Report::build(ReportKind::Error, open.source().to_owned(), open.start())
                    .with_code("P-003")
                    .with_message("Unclosed Brace")
                    .with_label(
                        Label::new(open)
                            .with_message(format!("unclosed opening brace found here"))
                            .with_color(Color::Red),
                    )
                    .with_label(
                        Label::new(end)
                            .with_message(format!("expected closing brace by this point"))
                            .with_color(Color::Cyan),
                    )
            }
            ParseError::InlineError {
                block_source,
                inline_source,
            } => Report::build(
                ReportKind::Error,
                inline_source.source().to_owned(),
                inline_source.start(),
            )
            .with_code("P-004")
            .with_message("Inline Error")
            .with_label(
                Label::new(block_source)
                    .with_message("multi-line block not allowed here, use '=>' instead")
                    .with_color(Color::Red),
            )
            .with_label(
                Label::new(inline_source)
                    .with_message("the '=>' token forces its statement to be inline")
                    .with_color(Color::Cyan),
            ),
            ParseError::EmptyBlock { source } => Report::build(
                ReportKind::Error,
                source.source().to_owned(),
                source.start(),
            )
            .with_code("P-005")
            .with_message("Empty Block")
            .with_label(
                Label::new(source)
                    .with_message("expected statement, found an empty block")
                    .with_color(Color::Red),
            )
            .with_note("try putting a temporary 'none' on the next line"),
        }
        .finish()
    }
}
