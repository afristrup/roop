use crate::{CliError, HybridArgs, HybridEmit};
use roop_hybrid::{Limits, compile, emit_ir, emit_qasm, execute, report};

/// `roop hybrid`: compiles an HQbricks∞ program to OpenQASM 3 or hybrid IR,
/// or runs it exactly and reports termination and expected costs.
pub fn run_hybrid(args: &HybridArgs) -> Result<(), CliError> {
    let name = args.input.display().to_string();
    let text = std::fs::read_to_string(&args.input).map_err(|e| CliError::Io(name.clone(), e))?;
    let fail = |e: roop_hybrid::HybridError| CliError::Tool(format!("{name}: {e}"));
    let program = compile(&text, args.optimized).map_err(fail)?;
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
