use boba_script_ast::int::IBig;
use derive_more::derive::Display;

#[derive(Debug, Display, Clone, Copy)]
#[display("{source}")]
pub struct IntBuilder<'a> {
    source: &'a str,
}

impl<'a> IntBuilder<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn build(&self) -> IBig {
        self.source.parse().expect("valid int")
    }
}

#[derive(Debug, Display, Clone, Copy)]
#[display("{source}")]
pub struct FloatBuilder<'a> {
    source: &'a str,
}

impl<'a> FloatBuilder<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn build(&self) -> f64 {
        self.source.parse().expect("valid float")
    }
}

#[derive(Debug, Display, Clone, Copy)]
#[display("'{source}'")]
pub struct StrFormat<'a> {
    source: &'a str,
}

impl<'a> StrFormat<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn format(&self) -> String {
        // TODO: parse escapes
        self.source.to_string()
    }
}
