use crate::{CodegenError, FnGen, Slot, llvm_type, ptr_type};

pub fn mem_store(g: &mut FnGen, slot: &Slot, value: &str) -> Result<(), CodegenError> {
    let ty = llvm_type(g.ctx, &slot.ty)?;
    let ptr = ptr_type(g.dialect, &ty, slot.space);
    g.emit(&format!("store {ty} {value}, {ptr} {}", slot.addr));
    Ok(())
}
