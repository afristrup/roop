/// What a failed assertion does in the code being generated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbortMode {
    /// Stop the program.
    Trap,
    /// Inside a `try` body: jump to the rollback code.
    Label(String),
    /// Inside a task or loop body launched from a `try`: raise the shared
    /// abort flag and return, leaving the launcher to notice.
    Flag,
}
