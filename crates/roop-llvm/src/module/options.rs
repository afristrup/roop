use crate::ParallelOptions;

/// Whole-program code generation settings.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// LLVM target triple, e.g. `arm64-apple-macosx`.
    pub triple: Option<String>,
    /// LLVM CPU name, e.g. `apple-m4`.
    pub cpu: Option<String>,
    pub parallel: ParallelOptions,
    /// A C `main` is linked in, so none is made for the roop `main`.
    pub no_entry: bool,
    /// The most bytes the history of kept values may hold, set by the program's
    /// entry point. Unlimited when absent.
    pub history_limit: Option<u64>,
}
