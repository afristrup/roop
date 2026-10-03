use std::path::PathBuf;

pub struct FmtArgs {
    /// Files and directories; empty means the whole project.
    pub paths: Vec<PathBuf>,
    pub check: bool,
    pub width: Option<usize>,
    pub stdin: bool,
}
