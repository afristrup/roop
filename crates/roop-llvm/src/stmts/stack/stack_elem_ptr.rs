use crate::{FnGen, StackParts};

pub fn stack_elem_ptr(g: &mut FnGen, stack: &StackParts, index: &str) -> String {
    let ptr = format!("%{}", g.fresh("t"));
    g.emit(&format!(
        "{ptr} = getelementptr inbounds {}, ptr {}, i32 0, i32 1, i64 {index}",
        stack.ty, stack.addr
    ));
    ptr
}
