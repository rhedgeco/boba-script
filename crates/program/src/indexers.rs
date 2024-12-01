macro_rules! build_indexers {
    {$($Indexer:ident),* $(,)?} => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $Indexer(usize);
            impl $Indexer {
                pub fn new(raw: usize) -> Self {
                    Self(raw)
                }

                pub fn raw(&self) -> usize {
                    self.0
                }
            }
        )*
    };
}

build_indexers! {
    ScopeIndex,
    ClassIndex,
    FuncIndex,
}

impl ScopeIndex {
    pub fn root() -> Self {
        ScopeIndex(0)
    }
}
