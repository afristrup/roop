use crate::{CliError, HybridArgs, HybridEmit, config_for, load_source};
use roop_hybrid::{Limits, compile, emit_ir, emit_qasm, execute, lower_roop, optimize, report};

/// `roop hybrid`: compiles an HQbricks∞ program, or a roop program with
/// qubits, to OpenQASM 3 or hybrid IR, or runs it exactly and reports
/// termination and expected costs.
pub fn run_hybrid(args: &HybridArgs) -> Result<(), CliError> {
    let name = args.input.display().to_string();
    let fail = |e: roop_hybrid::HybridError| CliError::Tool(format!("{name}: {e}"));
    let program = match args.input.extension().is_some_and(|e| e == "roop") {
        true => {
            let source = load_source(&args.input, &config_for(&args.input)?)?;
            roop_check::check(&source).map_err(CliError::Check)?;
            let program = lower_roop(&source).map_err(fail)?;
            match args.optimized {
                true => optimize(program),
                false => program,
            }
        }
        false => {
            let text =
                std::fs::read_to_string(&args.input).map_err(|e| CliError::Io(name.clone(), e))?;
            compile(&text, args.optimized).map_err(fail)?
        }
    };
    let out = match args.emit {
        HybridEmit::Qasm => emit_qasm(&program),
        HybridEmit::Ir => emit_ir(&program),
        HybridEmit::Report => {
            let mut limits = Limits::default();
            limits.iterations = args.iterations.unwrap_or(limits.iterations);
            report(&program, &execute(&program, &limits).map_err(fail)?)
        }
    };
    match &args.output {
        Some(path) => {
            std::fs::write(path, out).map_err(|e| CliError::Io(path.display().to_string(), e))
        }
        None => {
            print!("{out}");
            Ok(())
        }
    }
}
