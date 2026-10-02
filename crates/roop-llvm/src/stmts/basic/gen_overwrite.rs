use crate::{
    CodegenError, Dir, FnGen, Kind, gen_expr, gen_place, kind_of, llvm_type, mem_load, mem_store,
    same_type,
};
use roop_syntax::{Expr, OverwriteOp, Place};

/// `x = e` and friends destroy the old value, so they only exist going
/// forward; the function containing one has no inverse.
pub fn gen_overwrite(
    g: &mut FnGen,
    target: &Place,
    op: OverwriteOp,
    value: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    if dir == Dir::Backward {
        return Err(CodegenError::Unsupported("running an overwrite backward"));
    }
    let slot = gen_place(g, target)?;
    let v = gen_expr(g, value)?;
    same_type(&slot.ty, &v.ty)?;
    if op == OverwriteOp::Assign {
        return mem_store(g, &slot, &v.reg);
    }
    let kind = kind_of(g.ctx, &slot.ty)?;
    let instr = match (kind, op) {
        (Kind::Int, OverwriteOp::Mul) => "mul",
        (Kind::Int, OverwriteOp::Div) => "sdiv",
        (Kind::Int, OverwriteOp::Rem) => "srem",
        (Kind::Float, OverwriteOp::Mul) => "fmul",
        (Kind::Float, OverwriteOp::Div) => "fdiv",
        (Kind::Float, OverwriteOp::Rem) => "frem",
        _ => {
            return Err(CodegenError::InvalidOperand(
                "overwrite not defined for type",
            ));
        }
    };
    let ty = llvm_type(g.ctx, &slot.ty)?;
    let old = mem_load(g, &slot)?;
    let new = format!("%{}", g.fresh("t"));
    g.emit(&format!("{new} = {instr} {ty} {}, {}", old.reg, v.reg));
    mem_store(g, &slot, &new)
}
