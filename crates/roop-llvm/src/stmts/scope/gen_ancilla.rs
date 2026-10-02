use crate::{CodegenError, Dir, FnGen, Slot, gen_block, gen_expr, llvm_type, mem_store, same_type};
use roop_syntax::{Block, Expr, Type};

pub fn gen_ancilla(
    g: &mut FnGen,
    name: &str,
    ty: &Type,
    init: &Expr,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
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
    g.vars.push((name.into(), slot));
    let result = gen_block(g, body, dir);
    g.vars.pop();
    result
}
