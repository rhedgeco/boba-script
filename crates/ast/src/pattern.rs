use dashu_int::IBig;

#[derive(Debug, Clone)]
pub enum Pattern {
    Var(String),
    Bool(bool),
    Int(IBig),
    Float(f64),
    String(String),
}
