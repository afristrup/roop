use crate::{CodegenError, FnGen, Slot, StackParts, llvm_type, mem_load, push_value};

/// Moves the value at `place` onto the stack and leaves zero behind.
pub fn move_onto(g: &mut FnGen, parts: &StackParts, place: &Slot) -> Result<(), CodegenError> {
    let value = mem_load(g, place)?;
    push_value(g, parts, &value)?;
    let ty = llvm_type(g.ctx, &place.ty)?;
    g.emit(&format!("store {ty} zeroinitializer, ptr {}", place.addr));
    Ok(())
}
