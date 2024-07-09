use ariadne::{Color, Label, Report, ReportKind, Span};
use boba_script::{core::engine::EvalError, lexer::cache::CacheSpan};

pub trait ToAriadne {
    fn to_ariadne(&self) -> Report<CacheSpan>;
}

impl ToAriadne for EvalError<CacheSpan> {
    fn to_ariadne(&self) -> Report<CacheSpan> {
        match self {
            EvalError::UnknownVariable { name, data } => {
                Report::build(ReportKind::Error, data.id, data.start())
                    .with_code("R-001")
                    .with_message("Unknown Variable")
                    .with_label(
                        Label::new(data.clone())
                            .with_message(format!("unknown variable {}", name))
                            .with_color(Color::Red),
                    )
            }
            EvalError::InvalidUnaryOp { ty, op, data } => {
                Report::build(ReportKind::Error, data.id, data.start())
                    .with_code("R-002")
                    .with_message("Invalid Unary Operator")
                    .with_label(
                        Label::new(data.clone())
                            .with_message(format!(
                                "'{}' operator is not valid for '{}' types",
                                op, ty
                            ))
                            .with_color(Color::Red),
                    )
            }
            EvalError::InvalidBinaryOp { ty1, ty2, op, data } => {
                Report::build(ReportKind::Error, data.id, data.start())
                    .with_code("R-003")
                    .with_message("Invalid Binary Operator")
                    .with_label(
                        Label::new(data.clone())
                            .with_message(format!(
                                "'{}' does not have a valid '{}' operator for '{}' types",
                                ty1, op, ty2
                            ))
                            .with_color(Color::Red),
                    )
            }
            EvalError::InvalidAssign { data } => {
                Report::build(ReportKind::Error, data.id, data.start())
                    .with_code("R-004")
                    .with_message("Invalid Assignment")
                    .with_label(
                        Label::new(data.clone())
                            .with_message(format!("cannot assign to this expression"))
                            .with_color(Color::Red),
                    )
            }
            EvalError::InvalidTupleSize {
                lhs_count,
                rhs_count,
                lhs_data,
                rhs_data,
            } => Report::build(ReportKind::Error, rhs_data.id, rhs_data.start())
                .with_code("R-005")
                .with_message("Invalid Tuple Destructure")
                .with_label(
                    Label::new(lhs_data.clone())
                        .with_message(format!(
                            "expected tuple with {} parameters, found {}",
                            rhs_count, lhs_count
                        ))
                        .with_color(Color::Red),
                )
                .with_label(
                    Label::new(rhs_data.clone())
                        .with_message(format!("this is a tuple with {} parameters", rhs_count))
                        .with_color(Color::Cyan),
                ),
            EvalError::InvalidTupleDestructure {
                lhs_count,
                lhs_data,
                rhs_data,
            } => Report::build(ReportKind::Error, rhs_data.id, rhs_data.start())
                .with_code("R-006")
                .with_message("Invalid Tuple Destructure")
                .with_label(
                    Label::new(lhs_data.clone())
                        .with_message(format!(
                            "cannot destructure into tuple with {} params",
                            lhs_count
                        ))
                        .with_color(Color::Red),
                )
                .with_label(
                    Label::new(rhs_data.clone())
                        .with_message(format!("this expression produces a single value"))
                        .with_color(Color::Cyan),
                ),
            EvalError::UnexpectedType {
                expect,
                found,
                data,
            } => Report::build(ReportKind::Error, data.id, data.start())
                .with_code("R-007")
                .with_message("Unexpected Type")
                .with_label(
                    Label::new(data.clone())
                        .with_message(format!("expected '{}', found '{}'", expect, found))
                        .with_color(Color::Red),
                ),
        }
        .finish()
    }
}
