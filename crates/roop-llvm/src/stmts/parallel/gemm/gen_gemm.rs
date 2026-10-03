use crate::{
    CodegenError, Dir, FnGen, gen_place, match_gemm, matrix_dims, mem_load, mem_store,
    parallel_prologue, signed_factor,
};
use roop_syntax::{Block, Expr, Place};

/// Runs a `dgemm` loop nest on the runtime's matrix kernel when the CPU has
/// one, and returns whether it did. Backward the kernel subtracts, which undoes
/// the forward product. Where the loops would end with `i` at its last value,
/// so does this.
pub fn gen_gemm(
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
    let Some(gemm) = match_gemm(entry, body, step, until) else {
        return Ok(false);
    };
    let place = |g: &mut FnGen, name: &str| gen_place(g, &Place::Var(name.into()));
    let (c, a, b) = (place(g, gemm.c)?, place(g, gemm.a)?, place(g, gemm.b)?);
    let shapes = (matrix_dims(&c.ty), matrix_dims(&a.ty), matrix_dims(&b.ty));
    let expected = (
        Some((gemm.rows, gemm.cols)),
        Some((gemm.rows, gemm.inner)),
        Some((gemm.inner, gemm.cols)),
    );
    if shapes != expected {
        return Ok(false);
    }
    let loop_state = parallel_prologue(g, entry, step, until, dir)?;
    let alpha = place(g, gemm.alpha)?;
    let alpha = mem_load(g, &alpha)?;
    let factor = signed_factor(g, &alpha.reg, dir);
    g.emit(&format!(
        "call void @roop_dgemm(ptr {}, ptr {}, ptr {}, double {factor}, i64 {}, i64 {}, i64 {})",
        c.addr, a.addr, b.addr, gemm.rows, gemm.cols, gemm.inner
    ));
    mem_store(g, &loop_state.slot, &loop_state.end.reg)?;
    Ok(true)
}
