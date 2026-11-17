use crate::{CliError, HybridArgs, HybridEmit, USAGE};
use std::path::PathBuf;

pub fn parse_hybrid_args(mut args: impl Iterator<Item = String>) -> Result<HybridArgs, CliError> {
    let usage = || CliError::Usage(USAGE.into());
    let (mut input, mut output, mut emit) = (None, None, HybridEmit::Qasm);
    let (mut optimized, mut iterations) = (true, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => output = Some(PathBuf::from(args.next().ok_or_else(usage)?)),
            "-O0" => optimized = false,
            "--emit" => {
                emit = match args.next().ok_or_else(usage)?.as_str() {
                    "qasm" => HybridEmit::Qasm,
                    "ir" => HybridEmit::Ir,
                    "report" => HybridEmit::Report,
                    _ => return Err(usage()),
                }
            }
            "--iterations" => {
                let n = args.next().and_then(|n| n.parse().ok());
                iterations = Some(n.ok_or_else(usage)?);
            }
            flag if flag.starts_with('-') => return Err(usage()),
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(usage()),
        }
    }
    Ok(HybridArgs {
        input: input.ok_or_else(usage)?,
        output,
        emit,
        optimized,
        iterations,
    })
}
