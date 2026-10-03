use crate::{BuildArgs, CliError, Emit, build_program, run_exe, trace_driver, trace_report};
use roop_syntax::{FnDef, Program};
use std::path::Path;
use std::time::Duration;

/// Runs a failing test again a statement at a time and says where it stopped
/// and what the state was before each statement. Because every statement is
/// reversible, the states before are all there is to know.
pub fn run_trace(
    config: &roop_config::Config,
    program: &Program,
    test: &FnDef,
    src: &str,
    dir: &Path,
    timeout: Duration,
) -> Result<String, CliError> {
    let (staged, stages) = roop_opt::split_test(program, &test.name)
        .ok_or_else(|| CliError::Tool(format!("test {} not found", test.name)))?;
    let driver = dir.join("trace.c");
    std::fs::write(&driver, trace_driver(test, stages)?)
        .map_err(|e| CliError::Io(driver.display().to_string(), e))?;
    let exe = dir.join("trace");
    let args = BuildArgs {
        input: dir.join("unused.roop"),
        output: exe.clone(),
        emit: Emit::Executable,
        link: vec![driver],
        keep_tests: true,
    };
    build_program(&args, config, &staged)?;
    let ran = run_exe(&exe, &[], timeout)
        .map_err(|e| CliError::Io(format!("tracing {}", test.name), e))?;
    let finished = ran.code == Some(0) && !ran.timed_out;
    Ok(trace_report(src, test, &ran.stdout, !finished))
}
