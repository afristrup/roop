use super::{CodegenError, FnGen, Kind, Value, kind_of, llvm_type};
use roop_syntax::UnOp;

pub fn gen_unary(g: &mut FnGen, op: UnOp, v: Value) -> Result<Value, CodegenError> {
    let kind = kind_of(g.ctx, &v.ty)?;
    let ty = llvm_type(g.ctx, &v.ty)?;
    let reg = format!("%{}", g.fresh("t"));
    let instr = match (op, kind) {
        (UnOp::Neg, Kind::Int) => format!("sub {ty} 0, {}", v.reg),
        (UnOp::Neg, Kind::Float) => format!("fneg {ty} {}", v.reg),
        (UnOp::Not, Kind::Bool) => format!("xor {ty} {}, true", v.reg),
        _ => return Err(CodegenError::InvalidOperand("unary operator on wrong type")),
    };
    g.emit(&format!("{reg} = {instr}"));
    Ok(Value { reg, ty: v.ty })
}
