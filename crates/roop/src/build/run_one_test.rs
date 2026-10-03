use crate::run_exe;
use std::path::Path;
use std::time::Duration;

/// How a test ended.
pub enum Verdict {
    Passed,
    /// The backward run left something that was not zero.
    NotRestored(String),
    /// A check failed in the forward or backward run, or the test hit a trap.
    Failed,
    TimedOut,
}

/// Runs test `index` of the compiled test program in a process of its own, so
/// a failing test cannot take the others with it.
pub fn run_one_test(exe: &Path, index: usize, timeout: Duration) -> std::io::Result<Verdict> {
    let ran = run_exe(exe, &[index.to_string()], timeout)?;
    Ok(match (ran.timed_out, ran.code) {
        (true, _) => Verdict::TimedOut,
        (_, Some(0)) => Verdict::Passed,
        (_, Some(3)) => Verdict::NotRestored(ran.stdout.trim().to_string()),
        _ => Verdict::Failed,
    })
}
