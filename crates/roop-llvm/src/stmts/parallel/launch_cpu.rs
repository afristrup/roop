use crate::{CodegenError, Dir, FnGen, ParallelLoop, capture_env, check_abort, outline_block};
use roop_syntax::Block;

/// Hands the iterations to the CPU thread pool.
pub fn launch_cpu(
    g: &mut FnGen,
    pl: &ParallelLoop,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    let env = capture_env(g);
    let symbol = outline_block(g, Some(&pl.var), body, dir, "par")?;
    g.emit(&format!(
        "call void @roop_parallel_for(i64 {}, i64 {}, i64 {}, ptr @{symbol}, ptr {env})",
        pl.space.lo.reg, pl.space.count, pl.step
    ));
    check_abort(g)
}
