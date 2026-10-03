use crate::{CliError, FmtArgs, USAGE};
use std::path::PathBuf;

pub fn parse_fmt_args(mut args: impl Iterator<Item = String>) -> Result<FmtArgs, CliError> {
    let usage = || CliError::Usage(USAGE.into());
    let mut parsed = FmtArgs {
        paths: Vec::new(),
        check: false,
        width: None,
        stdin: false,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => parsed.check = true,
            "--stdin" => parsed.stdin = true,
            "--width" => {
                let width = args.next().and_then(|w| w.parse().ok());
                parsed.width = Some(width.ok_or_else(usage)?);
            }
            flag if flag.starts_with('-') => return Err(usage()),
            _ => parsed.paths.push(PathBuf::from(arg)),
        }
    }
    Ok(parsed)
}
