use crate::{
    CodegenError, Dir, FnGen, gen_place, match_int_matmul, matrix_dims, mem_store,
    parallel_prologue,
};
use roop_syntax::{Block, Expr, Place};

/// Runs an integer matrix product on the runtime's kernel, and returns whether
/// it did: the `q12` ones (each product divided by 4096) on the NEON kernel, and
/// the `i64` ones on the SME matrix unit when the CPU has one. A kernel returns
/// what the loops would, to the bit, and backward it takes off what forward
/// added.
pub fn gen_int_matmul(
    g: &mut FnGen,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<bool, CodegenError> {
    let Some(product) = match_int_matmul(entry, body, step, until) else {
        return Ok(false);
    };
    let (kernel, available) = match product.scaled {
        true => ("roop_q12_matmul", g.ctx.options.q12),
        false => ("roop_i64_matmul", g.ctx.options.sme),
    };
    if !available {
        return Ok(false);
    }
    let place = |g: &mut FnGen, name: &str| gen_place(g, &Place::Var(name.into()));
    let (c, a, b) = (
        place(g, product.c)?,
        place(g, product.a)?,
        place(g, product.b)?,
    );
    let shapes = [&c, &a, &b].map(|p| matrix_dims(&p.ty, "i64"));
    if shapes != product.shapes().map(Some) {
        return Ok(false);
    }
    let loop_state = parallel_prologue(g, entry, step, until, dir)?;
    let sign = match dir {
        Dir::Forward => 1,
        Dir::Backward => -1,
    };
    g.emit(&format!(
        "call void @{kernel}(ptr {}, ptr {}, ptr {}, i64 {sign}, i64 {}, i64 {}, i64 {}, i64 {})",
        c.addr,
        a.addr,
        b.addr,
        product.rows,
        product.cols,
        product.inner,
        product.layout.code()
    ));
    mem_store(g, &loop_state.slot, &loop_state.end.reg)?;
    Ok(true)
}
