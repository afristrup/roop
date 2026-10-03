use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
    let mut child = Command::new(exe)
        .arg(index.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > timeout {
            child.kill()?;
            child.wait()?;
            return Ok(Verdict::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let mut text = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        std::io::Read::read_to_string(&mut stdout, &mut text)?;
    }
    Ok(match status.code() {
        Some(0) => Verdict::Passed,
        Some(3) => Verdict::NotRestored(text.trim().to_string()),
        _ => Verdict::Failed,
    })
}
