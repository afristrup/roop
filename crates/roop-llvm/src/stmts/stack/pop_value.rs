use crate::{
    CodegenError, FnGen, Slot, StackParts, Value, bool_type, gen_assert, llvm_type, mem_load,
    stack_elem_ptr, stack_len_ptr,
};

/// Pops the top value and leaves zero in its slot; an empty stack is a failure.
pub fn pop_value(g: &mut FnGen, stack: &StackParts) -> Result<Value, CodegenError> {
    let len_ptr = stack_len_ptr(g, stack);
    let len = format!("%{}", g.fresh("t"));
    g.emit(&format!("{len} = load i64, ptr {len_ptr}"));
    let any = format!("%{}", g.fresh("t"));
    g.emit(&format!("{any} = icmp ne i64 {len}, 0"));
    gen_assert(
        g,
        &Value {
            reg: any,
            ty: bool_type(),
        },
    )?;
    let top = format!("%{}", g.fresh("t"));
    g.emit(&format!("{top} = sub i64 {len}, 1"));
    let elem_ptr = stack_elem_ptr(g, stack, &top);
    let slot = Slot {
        addr: elem_ptr.clone(),
        ty: stack.elem.clone(),
        space: 0,
    };
    let value = mem_load(g, &slot)?;
    let ty = llvm_type(g.ctx, &stack.elem)?;
    g.emit(&format!("store {ty} zeroinitializer, ptr {elem_ptr}"));
    g.emit(&format!("store i64 {top}, ptr {len_ptr}"));
    Ok(value)
}
