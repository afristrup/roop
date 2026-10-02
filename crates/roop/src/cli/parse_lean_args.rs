use crate::{CliError, LeanArgs, USAGE};
use std::path::PathBuf;

pub fn parse_lean_args(mut args: impl Iterator<Item = String>) -> Result<LeanArgs, CliError> {
    let usage = || CliError::Usage(USAGE.into());
    let (mut input, mut output, mut check) = (None, None, false);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => output = Some(PathBuf::from(args.next().ok_or_else(usage)?)),
            "--check" => check = true,
            flag if flag.starts_with('-') => return Err(usage()),
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(usage()),
        }
    }
    let input = input.ok_or_else(usage)?;
    let output = output.unwrap_or_else(|| input.with_extension("lean"));
    Ok(LeanArgs { input, output, check })
}
