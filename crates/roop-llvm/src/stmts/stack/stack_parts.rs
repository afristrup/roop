use crate::{CodegenError, FnGen, Slot, llvm_type};
use roop_syntax::Type;

/// A stack in memory: `{ len, [cap x elem] }`.
pub struct StackParts {
    pub addr: String,
    pub ty: String,
    pub elem: Type,
    pub cap: u64,
}

pub fn stack_parts(g: &FnGen, slot: &Slot) -> Result<StackParts, CodegenError> {
    let Type::Stack(elem, cap) = &slot.ty else {
        return Err(CodegenError::InvalidOperand("a stack place is required"));
    };
    Ok(StackParts {
        addr: slot.addr.clone(),
        ty: llvm_type(g.ctx, &slot.ty)?,
        elem: (**elem).clone(),
        cap: *cap,
    })
}
