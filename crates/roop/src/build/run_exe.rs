use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What a finished program left behind.
pub struct Ran {
    /// The exit code, or none when a signal ended it.
    pub code: Option<i32>,
    pub stdout: String,
    pub timed_out: bool,
}

/// Runs `exe`, and kills it when it takes longer than `timeout`.
pub fn run_exe(exe: &Path, args: &[String], timeout: Duration) -> std::io::Result<Ran> {
    let mut child = Command::new(exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut pipe = child.stdout.take().expect("stdout was piped");
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = std::io::Read::read_to_string(&mut pipe, &mut text);
        text
    });
    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > timeout {
            child.kill()?;
            timed_out = true;
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let stdout = reader.join().unwrap_or_default();
    Ok(Ran {
        code: status.code(),
        stdout,
        timed_out,
    })
}
