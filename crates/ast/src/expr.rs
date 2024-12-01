use dashu_int::IBig;

use crate::{class::ClassInit, node::BNode};

#[derive(Debug)]
pub enum Expr {
    // values
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Structure(ClassInit),
    Var(String),

    // unary ops
    Neg(BNode<Expr>),
    Pos(BNode<Expr>),
    Not(BNode<Expr>),

    // binary ops
    Add(BNode<Expr>, BNode<Expr>),
    Sub(BNode<Expr>, BNode<Expr>),
    Mul(BNode<Expr>, BNode<Expr>),
    Div(BNode<Expr>, BNode<Expr>),
    Mod(BNode<Expr>, BNode<Expr>),
    Pow(BNode<Expr>, BNode<Expr>),
}
