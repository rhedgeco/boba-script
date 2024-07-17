use std::{fmt::Display, ops::Deref};

use super::{Value, ValueKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple<Source> {
    items: Box<[Value<Source>]>,
}

impl<Source> Display for Tuple<Source> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self
            .items
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "({items})")
    }
}

impl<Source> FromIterator<Value<Source>> for Tuple<Source> {
    fn from_iter<T: IntoIterator<Item = Value<Source>>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl<Source> Tuple<Source> {
    pub fn kind(&self) -> TupleKind {
        self.items.deref().into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleKind {
    items: Box<[ValueKind]>,
}

impl<'a, Source> From<&'a [Value<Source>]> for TupleKind {
    fn from(values: &'a [Value<Source>]) -> Self {
        Self {
            items: values.iter().map(|v| v.kind()).collect(),
        }
    }
}

impl Display for TupleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self
            .items
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "({items})")
    }
}

impl TupleKind {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn items(&self) -> &[ValueKind] {
        &self.items
    }
}
