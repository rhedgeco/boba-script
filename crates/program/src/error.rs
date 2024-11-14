#[derive(Debug)]
pub enum BuildError {
    EmptyPath,
    SuperFromRoot,
    InvalidRootKeyword,
    ModuleDoesNotExist(String),
    PearlDoesNotExist(String),
}
