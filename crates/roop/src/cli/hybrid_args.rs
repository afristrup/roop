use crate::HybridEmit;
use std::path::PathBuf;

pub struct HybridArgs {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub emit: HybridEmit,
    pub optimized: bool,
    pub iterations: Option<usize>,
}
