use crate::Emit;
use std::path::PathBuf;

pub struct BuildArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    pub emit: Emit,
    pub link: Vec<PathBuf>,
    /// Keep the `test` items, which `roop test` runs.
    pub keep_tests: bool,
}
