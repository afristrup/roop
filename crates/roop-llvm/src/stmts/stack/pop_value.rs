use crate::{CodegenError, FnGen, StackParts, Value, drop_top, peek_top};

/// Pops the top value and leaves zero in its slot; an empty stack is a failure.
pub fn pop_value(g: &mut FnGen, stack: &StackParts) -> Result<Value, CodegenError> {
    let top = peek_top(g, stack)?;
    drop_top(g, stack, &top)?;
    Ok(top.value)
}
