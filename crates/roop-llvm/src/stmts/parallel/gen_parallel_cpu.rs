use crate::{CodegenError, Dir, FnGen, capture_env, mem_store, outline_body, parallel_prologue};
use roop_syntax::{Block, Expr};

/// Runs the iterations of a counted loop on CPU threads. Forward ends with
/// the induction variable at `hi`; backward ends with it at `lo`.
pub fn gen_parallel_cpu(
    g: &mut FnGen,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    let pl = parallel_prologue(g, entry, step, until, dir)?;
    let env = capture_env(g);
    let symbol = outline_body(g, &pl.var, body, dir)?;
    g.emit(&format!(
        "call void @roop_parallel_for(i64 {}, i64 {}, i64 {}, ptr @{symbol}, ptr {env})",
        pl.space.lo.reg, pl.space.count, pl.step
    ));
    mem_store(g, &pl.slot, &pl.end.reg)
}
