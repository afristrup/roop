use crate::ParallelOptions;

/// Whole-program code generation settings.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// LLVM target triple, e.g. `arm64-apple-macosx`.
    pub triple: Option<String>,
    /// LLVM CPU name, e.g. `apple-m4`.
    pub cpu: Option<String>,
    pub parallel: ParallelOptions,
}
