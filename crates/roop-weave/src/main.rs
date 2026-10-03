use roop_weave::{emit_model, parse_model};
use std::process::ExitCode;

const USAGE: &str = "usage: roop-weave <model.json> [-o <out.roop>] [--tests] [--batch <B>]";

fn run() -> Result<(), String> {
    let mut input = None;
    let mut output = None;
    let mut tests = false;
    let mut batch = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => output = Some(args.next().ok_or(USAGE)?),
            "--tests" => tests = true,
            "--batch" => {
                let size = args.next().ok_or(USAGE)?;
                batch = Some(
                    size.parse()
                        .map_err(|_| format!("--batch needs a number, not `{size}`"))?,
                );
            }
            _ if input.is_none() && !arg.starts_with('-') => input = Some(arg),
            _ => return Err(USAGE.into()),
        }
    }
    let input = input.ok_or(USAGE)?;
    let text = std::fs::read_to_string(&input).map_err(|e| format!("{input}: {e}"))?;
    let model = parse_model(&text).map_err(|e| format!("{input}: {e}"))?;
    let code = emit_model(&model, tests, batch);
    match output {
        Some(path) => std::fs::write(&path, code).map_err(|e| format!("{path}: {e}")),
        None => {
            print!("{code}");
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("roop-weave: {message}");
            ExitCode::FAILURE
        }
    }
}
