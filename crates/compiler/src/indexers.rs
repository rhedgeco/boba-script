macro_rules! build_indexers {
    {$($Indexer:ident),* $(,)?} => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $Indexer(usize);
            impl $Indexer {
                pub fn from_raw(index: usize) -> Self {
                    Self(index)
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
    FieldIndex,
}
