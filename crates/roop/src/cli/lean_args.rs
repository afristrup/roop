use std::path::PathBuf;

pub struct LeanArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    /// Also run Lean on the result and fail if it is rejected.
    pub check: bool,
}
