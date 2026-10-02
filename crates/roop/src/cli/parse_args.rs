use crate::{BuildArgs, CliError, Emit, USAGE};
use std::path::PathBuf;

pub fn parse_args(args: impl Iterator<Item = String>) -> Result<BuildArgs, CliError> {
    let usage = || CliError::Usage(USAGE.into());
    let mut args = args;
    if args.next().as_deref() != Some("build") {
        return Err(usage());
    }
    let (mut input, mut output, mut emit, mut link) = (None, None, None, Vec::new());
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => output = Some(PathBuf::from(args.next().ok_or_else(usage)?)),
            "--link" => link.push(PathBuf::from(args.next().ok_or_else(usage)?)),
            "--emit" => {
                emit = Some(match args.next().ok_or_else(usage)?.as_str() {
                    "ir" => Emit::Ir,
                    "obj" => Emit::Object,
                    "exe" => Emit::Executable,
                    _ => return Err(usage()),
                })
            }
            flag if flag.starts_with('-') => return Err(usage()),
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(usage()),
        }
    }
    let input = input.ok_or_else(usage)?;
    let emit = emit.unwrap_or(if link.is_empty() {
        Emit::Object
    } else {
        Emit::Executable
    });
    let extension = match emit {
        Emit::Ir => "ll",
        Emit::Object => "o",
        Emit::Executable => "",
    };
    let output = output.unwrap_or_else(|| input.with_extension(extension));
    Ok(BuildArgs {
        input,
        output,
        emit,
        link,
    })
}
