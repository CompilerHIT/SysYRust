#[derive(Debug)]
pub enum Error {
    Todo,
    VariableNotFound,
    PushPhiInGlobalDomain,
    MultipleDeclaration,
    FindVarError,
}
