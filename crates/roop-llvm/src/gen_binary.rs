use super::{CodegenError, FnGen, Value, binary_instr, bool_type, kind_of, llvm_type, same_type};
use roop_syntax::BinOp;

pub fn gen_binary(g: &mut FnGen, lhs: Value, op: BinOp, rhs: Value) -> Result<Value, CodegenError> {
    same_type(&lhs.ty, &rhs.ty)?;
    let kind = kind_of(g.ctx, &lhs.ty)?;
    let (instr, is_compare) = binary_instr(kind, op)?;
    let ty = llvm_type(g.ctx, &lhs.ty)?;
    let reg = format!("%{}", g.fresh("t"));
    g.emit(&format!("{reg} = {instr} {ty} {}, {}", lhs.reg, rhs.reg));
    let result = if is_compare { bool_type() } else { lhs.ty };
    Ok(Value { reg, ty: result })
}
