use derive_more::Display;

use super::value::ValueKind;

#[derive(Debug, Display, Clone)]
pub enum EvalError<Data> {
    #[display(fmt = "unknown variable '{}'", name)]
    UnknownVariable { name: String, data: Data },
    #[display(fmt = "'{}' operator is not valid for '{}' types", op, ty)]
    InvalidUnaryOp {
        ty: ValueKind,
        op: &'static str,
        data: Data,
    },
    #[display(
        fmt = "'{}' does not have a valid '{}' operator for '{}' types",
        ty1,
        op,
        ty2
    )]
    InvalidBinaryOp {
        ty1: ValueKind,
        ty2: ValueKind,
        op: &'static str,
        data: Data,
    },
    #[display(fmt = "cannot assign to this expression")]
    AssignError { data: Data },
    #[display(fmt = "expected '{}', found '{}'", expect, found)]
    UnexpectedType {
        expect: String,
        found: ValueKind,
        data: Data,
    },
}
