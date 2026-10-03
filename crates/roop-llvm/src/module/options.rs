use crate::ParallelOptions;

/// Whole-program code generation settings.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// LLVM target triple, e.g. `arm64-apple-macosx`.
    pub triple: Option<String>,
    /// LLVM CPU name, e.g. `apple-m4`.
    pub cpu: Option<String>,
    pub parallel: ParallelOptions,
    /// The CPU has the SME matrix unit, so a `dgemm` loop nest calls the
    /// runtime's matrix kernel instead of being compiled as loops.
    pub sme: bool,
    /// A C `main` is linked in, so none is made for the roop `main`.
    pub no_entry: bool,
    /// The most bytes the history of kept values may hold, set by the program's
    /// entry point. Unlimited when absent.
    pub history_limit: Option<u64>,
}
