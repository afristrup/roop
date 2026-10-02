use crate::{CodegenError, Dir, FnGen, Kind, gen_expr, gen_place, kind_of, llvm_type, same_type};
use roop_syntax::{Expr, Place, UpdateOp};

pub fn gen_update(
    g: &mut FnGen,
    target: &Place,
    op: UpdateOp,
    value: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    let slot = gen_place(g, target)?;
    let v = gen_expr(g, value)?;
    same_type(&slot.ty, &v.ty)?;
    let op = if dir == Dir::Backward {
        op.inverse()
    } else {
        op
    };
    let kind = kind_of(g.ctx, &slot.ty)?;
    let instr = match (kind, op) {
        (Kind::Int, UpdateOp::Add) => "add",
        (Kind::Int, UpdateOp::Sub) => "sub",
        (Kind::Int | Kind::Bool, UpdateOp::Xor) => "xor",
        (Kind::Float, UpdateOp::Add) => "fadd",
        (Kind::Float, UpdateOp::Sub) => "fsub",
        _ => return Err(CodegenError::InvalidOperand("update not defined for type")),
    };
    let ty = llvm_type(g.ctx, &slot.ty)?;
    let old = mem_load(g, &slot)?;
    let new = format!("%{}", g.fresh("t"));
    g.emit(&format!("{new} = {instr} {ty} {}, {}", old.reg, v.reg));
    mem_store(g, &slot, &new)
}
