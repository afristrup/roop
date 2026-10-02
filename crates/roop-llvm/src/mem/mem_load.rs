use crate::{CodegenError, FnGen, Slot, Value, llvm_type, ptr_type};

pub fn mem_load(g: &mut FnGen, slot: &Slot) -> Result<Value, CodegenError> {
    let ty = llvm_type(g.ctx, &slot.ty)?;
    let ptr = ptr_type(g.dialect, &ty, slot.space);
    let reg = format!("%{}", g.fresh("t"));
    g.emit(&format!("{reg} = load {ty}, {ptr} {}", slot.addr));
    Ok(Value { reg, ty: slot.ty.clone() })
}
