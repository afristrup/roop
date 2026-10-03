use crate::Dialect;

/// The type of a pointer to `llvm_ty` living in address space `space`.
pub fn ptr_type(dialect: Dialect, llvm_ty: &str, space: u32) -> String {
    match (dialect, space) {
        (Dialect::Air, 0) => format!("{llvm_ty}*"),
        (Dialect::Air, s) => format!("{llvm_ty} addrspace({s})*"),
        (_, 0) => "ptr".into(),
        (_, s) => format!("ptr addrspace({s})"),
    }
}
