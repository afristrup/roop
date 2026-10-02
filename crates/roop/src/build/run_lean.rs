use crate::{CliError, LeanArgs, config_for, find_lean, load_source, run_tool};
use std::process::Command;

/// `roop lean`: translate to Lean and report what the model proves.
pub fn run_lean(args: &LeanArgs) -> Result<(), CliError> {
    let program = load_source(&args.input, &config_for(&args.input)?)?;
    roop_check::check(&program).map_err(CliError::Check)?;
    let t = roop_lean::translate(&program);
    std::fs::write(&args.output, &t.lean)
        .map_err(|e| CliError::Io(args.output.display().to_string(), e))?;

    let list = |names: &[String]| names.join(", ");
    println!("wrote {}", args.output.display());
    println!(
        "  reversible, theorems proved by Lean: {}",
        list(&t.reversible)
    );
    if !t.one_way.is_empty() {
        println!(
            "  try: undone only on states the function produced: {}",
            list(&t.one_way)
        );
    }
    if !t.parallel.is_empty() {
        println!(
            "  parallel loops proved order-independent: {}",
            list(&t.parallel)
        );
    }
    if !t.inexact.is_empty() {
        println!(
            "  floating point, no roundtrip claimed: {}",
            list(&t.inexact)
        );
    }
    if !t.forward_only.is_empty() {
        println!(
            "  irreversible, forward model only: {}",
            list(&t.forward_only)
        );
    }
    for (name, why) in &t.skipped {
        println!("  skipped {name}: {why}");
    }
    if args.check {
        let lean = find_lean().ok_or_else(|| CliError::Tool("lean not found".into()))?;
        let mut command = Command::new(lean);
        command.arg(&args.output);
        run_tool(command)?;
        println!("  Lean accepted the file");
    }
    Ok(())
}
