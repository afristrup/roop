use crate::{
    CodegenError, FnGen, Slot, StackParts, Value, bool_type, gen_assert, mem_load, stack_elem_ptr,
    stack_len_ptr,
};

/// The top of a stack, read but not yet removed.
pub struct Top {
    pub value: Value,
    len_ptr: String,
    elem_ptr: String,
    index: String,
}

/// Reads the top value; an empty stack is a failure.
pub fn peek_top(g: &mut FnGen, stack: &StackParts) -> Result<Top, CodegenError> {
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
    let index = format!("%{}", g.fresh("t"));
    g.emit(&format!("{index} = sub i64 {len}, 1"));
    let elem_ptr = stack_elem_ptr(g, stack, &index);
    let slot = Slot {
        addr: elem_ptr.clone(),
        ty: stack.elem.clone(),
        space: 0,
    };
    let value = mem_load(g, &slot)?;
    Ok(Top {
        value,
        len_ptr,
        elem_ptr,
        index,
    })
}

/// Removes the value `peek_top` read, leaving zero in its slot.
pub fn drop_top(g: &mut FnGen, stack: &StackParts, top: &Top) -> Result<(), CodegenError> {
    let ty = crate::llvm_type(g.ctx, &stack.elem)?;
    g.emit(&format!("store {ty} zeroinitializer, ptr {}", top.elem_ptr));
    g.emit(&format!("store i64 {}, ptr {}", top.index, top.len_ptr));
    Ok(())
}
