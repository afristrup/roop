use crate::{FnGen, StackParts};

pub fn stack_len_ptr(g: &mut FnGen, stack: &StackParts) -> String {
    let ptr = format!("%{}", g.fresh("t"));
    g.emit(&format!(
        "{ptr} = getelementptr inbounds {}, ptr {}, i32 0, i32 0",
        stack.ty, stack.addr
    ));
    ptr
}
