use boba_script_ast::node::NodeId;
use derive_more::derive::Display;

use crate::value::ValueKind;

#[derive(Debug, Display)]
pub enum EvalError {
    #[display("invalid ast node")]
    InvalidNode(NodeId),
    #[display("unknown variable '{name}'")]
    UnknownVariable { id: NodeId, name: String },
    #[display("left hand side of assignment must be a variable")]
    InvalidAssignment(NodeId),
    #[display("no valid '{op}' operator for type '{kind}'")]
    InvalidUnaryOp {
        id: NodeId,
        op: String,
        kind: ValueKind,
    },
    #[display("type '{lhs}' has no valid '{op}' operator for type '{rhs}'")]
    InvalidBinaryOp {
        id: NodeId,
        op: String,
        lhs: ValueKind,
        rhs: ValueKind,
    },
    #[display("expected type '{expect}', found type '{found}'")]
    InvalidType {
        id: NodeId,
        expect: ValueKind,
        found: ValueKind,
    },
}
