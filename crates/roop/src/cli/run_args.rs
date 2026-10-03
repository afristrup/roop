use std::path::PathBuf;

pub struct RunArgs {
    pub input: PathBuf,
    /// What the program gets as its arguments.
    pub args: Vec<String>,
}
