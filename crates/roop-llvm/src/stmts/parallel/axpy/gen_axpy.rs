use crate::{
    CodegenError, Dir, FnGen, gen_place, match_axpy, mem_load, mem_store, parallel_prologue,
    signed_factor, vector_len,
};
use roop_syntax::{Block, Expr, Place};

/// The shortest vector worth the cost of entering streaming mode.
const MIN_LEN: i64 = 2048;

/// Runs a `daxpy` loop on the runtime's matrix kernel when the CPU has one,
/// and returns whether it did. Backward the kernel subtracts, which undoes the
/// forward update.
pub fn gen_axpy(
    g: &mut FnGen,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<bool, CodegenError> {
    if !g.ctx.options.sme {
        return Ok(false);
    }
    let Some(axpy) = match_axpy(entry, body, step, until).filter(|a| a.len >= MIN_LEN) else {
        return Ok(false);
    };
    let place = |g: &mut FnGen, name: &str| gen_place(g, &Place::Var(name.into()));
    let (y, x) = (place(g, axpy.y)?, place(g, axpy.x)?);
    if (vector_len(&y.ty), vector_len(&x.ty)) != (Some(axpy.len), Some(axpy.len)) {
        return Ok(false);
    }
    let loop_state = parallel_prologue(g, entry, step, until, dir)?;
    let alpha = place(g, axpy.alpha)?;
    let alpha = mem_load(g, &alpha)?;
    let factor = signed_factor(g, &alpha.reg, dir);
    g.emit(&format!(
        "call void @roop_daxpy(ptr {}, ptr {}, double {factor}, i64 {})",
        y.addr, x.addr, axpy.len
    ));
    mem_store(g, &loop_state.slot, &loop_state.end.reg)?;
    Ok(true)
}
