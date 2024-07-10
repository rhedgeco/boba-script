use std::{
    fmt::{Debug, Display},
    ops::Index,
    sync::atomic::{AtomicU32, Ordering},
};

use ariadne::{Cache, Source};
use boba_script_parser::{stream, token::Span};

/// Represents a range of bytes from a file stored in [`BobaCache`]
#[derive(Debug, Clone, Copy)]
pub struct CacheSpan {
    pub id: CacheId,
    pub span: Span,
}

impl stream::Source for CacheSpan {
    fn span(&self) -> Span {
        self.span
    }

    fn start(&self) -> usize {
        self.span.start
    }

    fn end(&self) -> usize {
        self.span.end
    }
}

impl ariadne::Span for CacheSpan {
    type SourceId = CacheId;

    fn source(&self) -> &Self::SourceId {
        &self.id
    }

    fn start(&self) -> usize {
        self.span.start
    }

    fn end(&self) -> usize {
        self.span.end
    }
}

impl CacheSpan {
    pub fn new(id: CacheId, span: impl Into<Span>) -> CacheSpan {
        Self {
            id,
            span: span.into(),
        }
    }
}

/// Handle to a specific location in a [`BobaCache`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheId(u64);

impl CacheId {
    pub fn span(&self, span: impl Into<Span>) -> CacheSpan {
        CacheSpan::new(*self, span)
    }

    fn build(id: u32, index: usize) -> Self {
        match u32::try_from(index) {
            Ok(index) => Self((id as u64) << 32 | (index as u64)),
            Err(_) => panic!("cache capacity overflow"),
        }
    }

    fn cache_id(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    fn uindex(&self) -> usize {
        (self.0 as u32) as usize
    }
}

#[derive(Debug)]
pub struct CacheData {
    label: String,
    source: Source,
    id: CacheId,
}

impl CacheData {
    pub fn id(&self) -> CacheId {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn text(&self) -> &str {
        self.source.text()
    }

    pub fn source(&self) -> &Source {
        &self.source
    }
}

/// Stores data and provides reference to it via a [`CacheId`]
#[derive(Debug)]
pub struct BobaCache {
    store: Vec<CacheData>,
    cache_id: u32,
}

impl Default for BobaCache {
    fn default() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let cache_id = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            cache_id,
            store: vec![],
        }
    }
}

impl Index<CacheId> for BobaCache {
    type Output = CacheData;

    fn index(&self, id: CacheId) -> &Self::Output {
        self.load(id).expect("invalid cache id")
    }
}

impl Cache<CacheId> for BobaCache {
    type Storage = String;

    fn fetch(&mut self, id: &CacheId) -> Result<&Source<Self::Storage>, Box<dyn Debug + '_>> {
        match self.load(*id) {
            Some(data) => Ok(data.source()),
            None => Err(Box::new("invalid cache id")),
        }
    }

    fn display<'a>(&self, id: &'a CacheId) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(self.load(*id)?.label().to_string()))
    }
}

impl BobaCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&self, id: CacheId) -> Option<&CacheData> {
        if id.cache_id() != self.cache_id {
            return None;
        }

        self.store.get(id.uindex())
    }

    pub fn store(&mut self, label: impl Into<String>, data: impl Into<String>) -> &CacheData {
        let id = CacheId::build(self.cache_id, self.store.len());
        self.store.push(CacheData {
            label: label.into(),
            source: Source::from(data.into()),
            id,
        });

        &self.store[id.uindex()]
    }
}
