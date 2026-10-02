use crate::{CodegenError, Dialect, Dir, FnGen, launch_gpu, mem_store, parallel_prologue};
use roop_syntax::{Block, Expr};

/// Launches the loop as a GPU kernel; the induction variable ends as in the
/// CPU version.
pub fn gen_parallel_gpu(
    g: &mut FnGen,
    dialect: Dialect,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    let pl = parallel_prologue(g, entry, step, until, dir)?;
    launch_gpu(g, dialect, &pl, body, dir)?;
    mem_store(g, &pl.slot, &pl.end.reg)
}
