#[derive(Debug, Clone)]
pub enum CompileError {
    ModuleAlreadyExists,
    ClassAlreadyExists,
    FuncAlreadyExists,
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
