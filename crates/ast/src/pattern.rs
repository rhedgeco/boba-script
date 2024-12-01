use dashu_int::IBig;

use crate::class::ClassPattern;

#[derive(Debug)]
pub enum Pattern {
    Var(String),
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
    Structure(ClassPattern),
}
