use crate::{
    CodegenError, Dir, FnGen, Value, apply_overwrite, bool_type, gen_assert, gen_binary, gen_expr,
    gen_place, kind_of, mem_load, mem_store, pop_value, push_value, same_type, stack_parts,
};
use roop_syntax::{BinOp, Expr, OverwriteOp, Place};

/// A destroying update inside `logged`. Forward pushes the old value and
/// stores the new one. Backward checks that the target still holds what the
/// update produced, pops the old value and puts it back.
pub fn gen_logged_overwrite(
    g: &mut FnGen,
    history: &Place,
    target: &Place,
    op: OverwriteOp,
    value: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    let history_slot = gen_place(g, history)?;
    let parts = stack_parts(g, &history_slot)?;
    let slot = gen_place(g, target)?;
    same_type(&parts.elem, &slot.ty)?;
    let v = gen_expr(g, value)?;
    same_type(&slot.ty, &v.ty)?;
    match dir {
        Dir::Forward => {
            if matches!(op, OverwriteOp::Div | OverwriteOp::Rem) {
                nonzero(g, &v)?;
            }
            let old = mem_load(g, &slot)?;
            push_value(g, &parts, &old)?;
            let new = apply_overwrite(g, op, &old, &v)?;
            mem_store(g, &slot, &new)
        }
        Dir::Backward => {
            let old = pop_value(g, &parts)?;
            let expected = apply_overwrite(g, op, &old, &v)?;
            let current = mem_load(g, &slot)?;
            let expected = Value {
                reg: expected,
                ty: slot.ty.clone(),
            };
            let same = gen_binary(g, current, BinOp::Eq, expected)?;
            gen_assert(g, &same)?;
            mem_store(g, &slot, &old.reg)
        }
    }
}

fn nonzero(g: &mut FnGen, divisor: &Value) -> Result<(), CodegenError> {
    let zero = match kind_of(g.ctx, &divisor.ty)? {
        crate::Kind::Float => "0.000000e+00",
        _ => "0",
    };
    let reg = format!("%{}", g.fresh("t"));
    let ty = crate::llvm_type(g.ctx, &divisor.ty)?;
    let compare = if zero == "0" { "icmp ne" } else { "fcmp one" };
    g.emit(&format!("{reg} = {compare} {ty} {}, {zero}", divisor.reg));
    gen_assert(
        g,
        &Value {
            reg,
            ty: bool_type(),
        },
    )
}
