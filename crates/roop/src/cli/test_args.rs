use std::path::PathBuf;

pub struct TestArgs {
    /// Files and directories; empty means the whole project.
    pub paths: Vec<PathBuf>,
    /// Only the tests whose name contains this text.
    pub filter: Option<String>,
    /// Seconds a test may run before it counts as failed.
    pub timeout: u64,
}
