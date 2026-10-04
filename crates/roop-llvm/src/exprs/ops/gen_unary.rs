use crate::{CodegenError, FnGen, Kind, Value, emit_int_arith, kind_of, llvm_type};
use roop_syntax::UnOp;

pub fn gen_unary(g: &mut FnGen, op: UnOp, v: Value) -> Result<Value, CodegenError> {
    let kind = kind_of(g.ctx, &v.ty)?;
    let ty = llvm_type(g.ctx, &v.ty)?;
    let reg = format!("%{}", g.fresh("t"));
    if (op, kind) == (UnOp::Neg, Kind::Int) {
        emit_int_arith(g, &reg, "sub", &ty, "0", &v.reg);
        return Ok(Value { reg, ty: v.ty });
    }
    let instr = match (op, kind) {
        (UnOp::Neg, Kind::Byte) => format!("sub {ty} 0, {}", v.reg),
        (UnOp::Neg, Kind::Float) => format!("fneg {ty} {}", v.reg),
        (UnOp::Not, Kind::Bool) => format!("xor {ty} {}, true", v.reg),
        _ => return Err(CodegenError::InvalidOperand("unary operator on wrong type")),
    };
    g.emit(&format!("{reg} = {instr}"));
    Ok(Value { reg, ty: v.ty })
}
