#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompileError {
    SuperFromRootScope,
    SuperInPath,
    ModuleDoesNotExist,
    ClassDoesNotExist,
    FuncDoesNotExist,
    PrivateModule,
    PrivateClass,
    PrivateField,
    PrivateFunc,
    EmptyPath,
}
