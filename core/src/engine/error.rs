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
    #[display(fmt = "failed to destructure expression into left hand side")]
    DestructureError { data: Data },
    #[display(
        fmt = "expected tuple with {} parameters, found {}",
        rhs_count,
        lhs_count
    )]
    TupleDestructureError {
        lhs_count: usize,
        rhs_count: usize,
        lhs_data: Data,
        rhs_data: Data,
    },
    #[display(fmt = "expected '{}', found '{}'", expect, found)]
    UnexpectedType {
        expect: ValueKind,
        found: ValueKind,
        data: Data,
    },
}
