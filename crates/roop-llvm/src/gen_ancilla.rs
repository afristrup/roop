use super::{CodegenError, Dir, FnGen, Slot, gen_block, gen_expr, llvm_type, same_type};
use roop_syntax::{Block, Expr, Type};

pub fn gen_ancilla(
    g: &mut FnGen,
    name: &str,
    ty: &Type,
    init: &Expr,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    let v = gen_expr(g, init)?;
    same_type(ty, &v.ty)?;
    let llvm_ty = llvm_type(g.ctx, ty)?;
    let addr = g.alloca(&llvm_ty);
    g.emit(&format!("store {llvm_ty} {}, ptr {addr}", v.reg));
    g.vars.push((
        name.into(),
        Slot {
            addr,
            ty: ty.clone(),
        },
    ));
    let result = gen_block(g, body, dir);
    g.vars.pop();
    result
}
