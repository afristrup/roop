use crate::Emit;
use std::path::PathBuf;

pub struct BuildArgs {
    pub input: PathBuf,
    /// Where to write; defaults to the input with the extension of what is
    /// emitted.
    pub output: Option<PathBuf>,
    /// What to write; an executable when there is a `main` or a `--link`, an
    /// object otherwise.
    pub emit: Option<Emit>,
    pub link: Vec<PathBuf>,
    /// Keep the `test` items, which `roop test` runs.
    pub keep_tests: bool,
}
