use crate::{CodegenError, FnGen};

/// Address of the innermost enclosing `try`'s abort flag.
pub fn abort_slot(g: &FnGen) -> Result<String, CodegenError> {
    g.vars
        .iter()
        .rev()
        .find(|(name, _)| name == "__abort")
        .map(|(_, slot)| slot.addr.clone())
        .ok_or(CodegenError::InvalidOperand("no enclosing try to abort"))
}
