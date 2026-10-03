use crate::{BuildArgs, CliError, Emit, RunArgs, build};
use std::process::Command;

/// `roop run`: builds the program, which needs a `main`, runs it with its
/// arguments and ends with its exit status.
pub fn run_run(args: &RunArgs) -> Result<(), CliError> {
    let dir = std::env::temp_dir().join(format!("roop-run-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| CliError::Io("temp dir".into(), e))?;
    let exe = dir.join("program");
    let built = build(&BuildArgs {
        input: args.input.clone(),
        output: Some(exe.clone()),
        emit: Some(Emit::Executable),
        link: Vec::new(),
        keep_tests: false,
    });
    let status = built.and_then(|()| {
        Command::new(&exe)
            .args(&args.args)
            .status()
            .map_err(|e| CliError::Io(exe.display().to_string(), e))
    });
    let _ = std::fs::remove_dir_all(&dir);
    std::process::exit(status?.code().unwrap_or(1));
}
