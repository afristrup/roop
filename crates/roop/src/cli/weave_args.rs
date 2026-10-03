use std::path::PathBuf;

pub struct WeaveArgs {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub tests: bool,
    pub main: bool,
    pub batch: Option<usize>,
    pub driver: Option<PathBuf>,
}
