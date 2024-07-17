use super::value::ValueKind;

#[derive(Debug, Clone)]
pub enum EvalError<Source> {
    UnknownVariable {
        name: String,
        source: Source,
    },
    InvalidUnaryOp {
        ty: ValueKind,
        op: &'static str,
        source: Source,
    },
    InvalidBinaryOp {
        ty1: ValueKind,
        ty2: ValueKind,
        op: &'static str,
        source: Source,
    },
    InvalidAssign {
        source: Source,
    },
    InvalidTupleSize {
        lhs_count: usize,
        rhs_count: usize,
        lhs_source: Source,
        rhs_source: Source,
    },
    InvalidTupleDestructure {
        lhs_count: usize,
        lhs_source: Source,
        rhs_source: Source,
    },
    UnexpectedType {
        expect: ValueKind,
        found: ValueKind,
        source: Source,
    },
    InvalidParameters {
        found: usize,
        expect: usize,
        source: Source,
    },
    NativeCall {
        message: String,
        source: Source,
    },
    UnknownFunction {
        name: String,
        source: Source,
    },
    NotAFunction {
        name: String,
        found: ValueKind,
        source: Source,
    },
}
