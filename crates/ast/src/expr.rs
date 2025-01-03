use dashu_int::IBig;

use crate::{class::ClassInit, Node};

#[derive(Debug, Clone)]
pub enum Expr<Type> {
    // values
    None,
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Var(String),
    Class(Node<ClassInit<Type>>),

    // unary ops
    Neg(Box<Node<Expr<Type>>>),
    Pos(Box<Node<Expr<Type>>>),
    Not(Box<Node<Expr<Type>>>),

    // binary ops
    Add(Box<(Node<Expr<Type>>, Node<Expr<Type>>)>),
    Sub(Box<(Node<Expr<Type>>, Node<Expr<Type>>)>),
    Mul(Box<(Node<Expr<Type>>, Node<Expr<Type>>)>),
    Div(Box<(Node<Expr<Type>>, Node<Expr<Type>>)>),
    Mod(Box<(Node<Expr<Type>>, Node<Expr<Type>>)>),
    Pow(Box<(Node<Expr<Type>>, Node<Expr<Type>>)>),
}
