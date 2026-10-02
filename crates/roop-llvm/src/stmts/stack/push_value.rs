use crate::{
    CodegenError, FnGen, Slot, StackParts, Value, bool_type, check_zero, gen_assert, mem_store, same_type, stack_elem_ptr, stack_len_ptr,
};

/// Pushes a value. The free slot must be zero, which every pop leaves behind,
/// so a push and a pop are exact inverses; a full stack is a failure.
pub fn push_value(g: &mut FnGen, stack: &StackParts, value: &Value) -> Result<(), CodegenError> {
    same_type(&stack.elem, &value.ty)?;
    let len_ptr = stack_len_ptr(g, stack);
    let len = format!("%{}", g.fresh("t"));
    g.emit(&format!("{len} = load i64, ptr {len_ptr}"));
    let room = format!("%{}", g.fresh("t"));
    g.emit(&format!("{room} = icmp ult i64 {len}, {}", stack.cap));
    gen_assert(
        g,
        &Value {
            reg: room,
            ty: bool_type(),
        },
    )?;
    let elem_ptr = stack_elem_ptr(g, stack, &len);
    let slot = Slot {
        addr: elem_ptr,
        ty: stack.elem.clone(),
        space: 0,
    };
    check_zero(g, &slot)?;
    mem_store(g, &slot, &value.reg)?;
    let next = format!("%{}", g.fresh("t"));
    g.emit(&format!("{next} = add i64 {len}, 1"));
    g.emit(&format!("store i64 {next}, ptr {len_ptr}"));
    Ok(())
}
