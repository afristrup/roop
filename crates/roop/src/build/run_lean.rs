use crate::{CliError, LeanArgs, find_lean, run_tool};
use std::process::Command;

/// `roop lean`: translate to Lean and report what the model proves.
pub fn run_lean(args: &LeanArgs) -> Result<(), CliError> {
    let source = std::fs::read_to_string(&args.input)
        .map_err(|e| CliError::Io(args.input.display().to_string(), e))?;
    let program = roop_syntax::parse(&source).map_err(CliError::Parse)?;
    roop_check::check(&program).map_err(CliError::Check)?;
    let t = roop_lean::translate(&program);
    std::fs::write(&args.output, &t.lean)
        .map_err(|e| CliError::Io(args.output.display().to_string(), e))?;

    let list = |names: &[String]| names.join(", ");
    println!("wrote {}", args.output.display());
    println!("  reversible, theorems proved by Lean: {}", count(&t.reversible, &t.open));
    if !t.open.is_empty() {
        println!("  open (loops need an induction): {}", list(&t.open));
    }
    if !t.inexact.is_empty() {
        println!("  floating point, no roundtrip claimed: {}", list(&t.inexact));
    }
    if !t.forward_only.is_empty() {
        println!("  irreversible, forward model only: {}", list(&t.forward_only));
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

fn count(reversible: &[String], open: &[String]) -> String {
    let proved: Vec<&String> = reversible
        .iter()
        .filter(|name| !open.iter().any(|o| o.starts_with(&format!("{name}_"))))
        .collect();
    format!("{} of {} functions", proved.len(), reversible.len())
}
