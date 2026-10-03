use crate::{CliError, RunArgs, USAGE};
use std::path::PathBuf;

/// `roop run <input.roop> [args...]`: everything after the input is the
/// program's.
pub fn parse_run_args(mut args: impl Iterator<Item = String>) -> Result<RunArgs, CliError> {
    let input = args
        .next()
        .filter(|a| !a.starts_with('-'))
        .ok_or_else(|| CliError::Usage(USAGE.into()))?;
    Ok(RunArgs {
        input: PathBuf::from(input),
        args: args.collect(),
    })
}
