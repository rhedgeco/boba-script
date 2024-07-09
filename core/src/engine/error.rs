use super::value::ValueKind;

#[derive(Debug, Clone)]
pub enum EvalError<Data> {
    UnknownVariable {
        name: String,
        data: Data,
    },
    InvalidUnaryOp {
        ty: ValueKind,
        op: &'static str,
        data: Data,
    },
    InvalidBinaryOp {
        ty1: ValueKind,
        ty2: ValueKind,
        op: &'static str,
        data: Data,
    },
    InvalidAssign {
        data: Data,
    },
    InvalidTupleSize {
        lhs_count: usize,
        rhs_count: usize,
        lhs_data: Data,
        rhs_data: Data,
    },
    InvalidTupleDestructure {
        lhs_count: usize,
        lhs_data: Data,
        rhs_data: Data,
    },
    UnexpectedType {
        expect: ValueKind,
        found: ValueKind,
        data: Data,
    },
}
