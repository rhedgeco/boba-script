use crate::parser::{PError, PResult, Span};

use super::span::Spanner;

pub fn parse_int(span: Span, str: impl AsRef<str>) -> PResult<Spanner<i64>> {
    match str.as_ref().parse() {
        Ok(value) => Ok(Spanner::new(span, value)),
        Err(error) => Err(PError::ParseIntError { error, span }),
    }
}

pub fn parse_float(span: Span, str: impl AsRef<str>) -> PResult<Spanner<f64>> {
    match str.as_ref().parse() {
        Ok(value) => Ok(Spanner::new(span, value)),
        Err(error) => Err(PError::ParseFloatError { error, span }),
    }
}
