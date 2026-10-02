use crate::{
    CodegenError, Dir, FnGen, apply_overwrite, gen_expr, gen_logged_overwrite, gen_place,
    mem_load, mem_store, same_type,
};
use roop_syntax::{Expr, OverwriteOp, Place};

/// `x = e` and friends destroy the old value. Outside a `logged` block they
/// only exist going forward, and the function containing one has no inverse.
/// Inside one, the old value goes on the history stack and the update can be
/// undone.
pub fn gen_overwrite(
    g: &mut FnGen,
    target: &Place,
    op: OverwriteOp,
    value: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    if let Some(history) = g.logged.last().cloned() {
        return gen_logged_overwrite(g, &history, target, op, value, dir);
    }
    if dir == Dir::Backward {
        return Err(CodegenError::Unsupported("running an overwrite backward"));
    }
    let slot = gen_place(g, target)?;
    let v = gen_expr(g, value)?;
    same_type(&slot.ty, &v.ty)?;
    let old = mem_load(g, &slot)?;
    let new = apply_overwrite(g, op, &old, &v)?;
    mem_store(g, &slot, &new)
}
