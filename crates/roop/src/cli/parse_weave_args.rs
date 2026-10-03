use crate::{CliError, USAGE, WeaveArgs};
use std::path::PathBuf;

pub fn parse_weave_args(mut args: impl Iterator<Item = String>) -> Result<WeaveArgs, CliError> {
    let usage = || CliError::Usage(USAGE.into());
    let (mut input, mut output, mut tests, mut batch) = (None, None, false, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => output = Some(PathBuf::from(args.next().ok_or_else(usage)?)),
            "--tests" => tests = true,
            "--batch" => {
                let size = args.next().and_then(|n| n.parse().ok());
                batch = Some(size.ok_or_else(usage)?);
            }
            flag if flag.starts_with('-') => return Err(usage()),
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(usage()),
        }
    }
    Ok(WeaveArgs {
        input: input.ok_or_else(usage)?,
        output,
        tests,
        batch,
    })
}
