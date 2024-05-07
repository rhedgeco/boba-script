use crate::token::Span;

pub type Color = ariadne::Color;

pub struct Label {
    pub message: String,
    pub color: Color,
    pub span: Span,
}

impl Label {
    pub fn new(message: impl Into<String>, color: Color, span: Span) -> Self {
        Self {
            message: message.into(),
            color,
            span,
        }
    }
}

pub struct LangError {
    pub message: String,
    pub labels: Vec<Label>,
}

impl LangError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            labels: Vec::new(),
        }
    }

    pub fn label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }
}
