use crate::{CodegenError, FnGen, Slot, gen_expr, llvm_type, mem_store, same_type};
use roop_syntax::{Expr, Type};

/// Allocates the ancilla and stores its start value; `empty` is a zeroed stack.
pub fn declare_ancilla(
    g: &mut FnGen,
    ty: &Type,
    init: &Expr,
) -> Result<Slot, CodegenError> {
    let llvm_ty = llvm_type(g.ctx, ty)?;
    let addr = g.alloca(&llvm_ty);
    let slot = Slot {
        addr,
        ty: ty.clone(),
        space: 0,
    };
    if *init == Expr::Empty {
        if !matches!(ty, Type::Stack(..)) {
            return Err(CodegenError::InvalidOperand("`empty` needs a stack type"));
        }
        mem_store(g, &slot, "zeroinitializer")?;
    } else {
        let v = gen_expr(g, init)?;
        same_type(ty, &v.ty)?;
        mem_store(g, &slot, &v.reg)?;
    }
    Ok(slot)
}
