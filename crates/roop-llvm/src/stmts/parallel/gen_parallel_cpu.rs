use crate::{CodegenError, Dir, FnGen, launch_cpu, mem_store, parallel_prologue};
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
    launch_cpu(g, &pl, body, dir)?;
    mem_store(g, &pl.slot, &pl.end.reg)
}
