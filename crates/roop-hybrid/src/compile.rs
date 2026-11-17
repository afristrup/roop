use crate::{HybridError, HybridProgram, check, lower, optimize, parse};

/// Source text to hybrid IR: parse, check well-formedness, lower, and
/// optimize unless `optimized` is false.
pub fn compile(text: &str, optimized: bool) -> Result<HybridProgram, HybridError> {
    let stmts = parse(text)?;
    let sig = check(&stmts)?;
    let program = lower(&stmts, &sig);
    Ok(match optimized {
        true => optimize(program),
        false => program,
    })
}
