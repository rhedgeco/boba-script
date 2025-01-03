use dashu_int::IBig;

use crate::{class::ClassInit, Node};

#[derive(Debug, Clone)]
pub enum Expr {
    // values
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Var(String),
    Class(Node<ClassInit>),

    // unary ops
    Neg(Box<Node<Expr>>),
    Pos(Box<Node<Expr>>),
    Not(Box<Node<Expr>>),

    // binary ops
    Add(Box<(Node<Expr>, Node<Expr>)>),
    Sub(Box<(Node<Expr>, Node<Expr>)>),
    Mul(Box<(Node<Expr>, Node<Expr>)>),
    Div(Box<(Node<Expr>, Node<Expr>)>),
    Mod(Box<(Node<Expr>, Node<Expr>)>),
    Pow(Box<(Node<Expr>, Node<Expr>)>),
}
