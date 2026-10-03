use std::path::PathBuf;

pub struct WeaveArgs {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub tests: bool,
    pub batch: Option<usize>,
}
