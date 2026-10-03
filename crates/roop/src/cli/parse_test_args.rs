use crate::{CliError, TestArgs, USAGE};
use std::path::PathBuf;

pub fn parse_test_args(mut args: impl Iterator<Item = String>) -> Result<TestArgs, CliError> {
    let usage = || CliError::Usage(USAGE.into());
    let mut parsed = TestArgs {
        paths: Vec::new(),
        filter: None,
        timeout: 60,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--filter" => parsed.filter = Some(args.next().ok_or_else(usage)?),
            "--timeout" => {
                let seconds = args.next().and_then(|s| s.parse().ok());
                parsed.timeout = seconds.ok_or_else(usage)?;
            }
            flag if flag.starts_with('-') => return Err(usage()),
            _ => parsed.paths.push(PathBuf::from(arg)),
        }
    }
    Ok(parsed)
}
