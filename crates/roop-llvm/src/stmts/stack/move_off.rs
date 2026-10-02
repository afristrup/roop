use crate::{CodegenError, FnGen, Slot, StackParts, check_zero, mem_store, pop_value, same_type};

/// Moves the top of the stack into `place`, which must be zero.
pub fn move_off(g: &mut FnGen, parts: &StackParts, place: &Slot) -> Result<(), CodegenError> {
    check_zero(g, place)?;
    let value = pop_value(g, parts)?;
    same_type(&place.ty, &value.ty)?;
    mem_store(g, place, &value.reg)
}
