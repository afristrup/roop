/// One name a `use` brings into scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    /// The name it has where it is imported.
    pub local: String,
    /// Its name in the whole program.
    pub global: String,
    /// The module it was found in.
    pub from: usize,
}
