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
    /// A product of `q12` matrices, the loops of weave's einsums, calls the
    /// runtime's integer kernel instead of being compiled as loops.
    pub q12: bool,
    /// An ancilla that a `call` made and an `uncall` of the same function takes
    /// off again, with nothing in between that changes what it reads, is set to
    /// zero instead of computed backward.
    pub clear_ancillas: bool,
    /// A C `main` is linked in, so none is made for the roop `main`.
    pub no_entry: bool,
    /// The most bytes the history of kept values may hold, set by the program's
    /// entry point. Unlimited when absent.
    pub history_limit: Option<u64>,
    /// Integer additions, subtractions, negations and multiplications on the
    /// host raise the sticky `roop_overflow` flag when they wrap. The results
    /// are unchanged.
    pub check_overflow: bool,
}
