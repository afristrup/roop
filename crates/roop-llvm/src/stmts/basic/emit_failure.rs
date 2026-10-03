use crate::{AbortMode, CodegenError, FnGen, abort_slot, ptr_type};

/// What a failed assertion does here. Hosts trap, unless a `try` is catching,
/// in which case it jumps to the rollback. Device kernels cannot trap; they
/// record the failure in the error buffer and stop.
pub fn emit_failure(g: &mut FnGen) -> Result<(), CodegenError> {
    match (g.error_flag.clone(), g.abort.clone()) {
        (Some(flag), _) => {
            let ptr = ptr_type(g.dialect, "i32", 1);
            g.emit(&format!("store i32 1, {ptr} {flag}"));
            g.emit("ret void");
        }
        (None, AbortMode::Trap) => {
            g.emit("call void @llvm.trap()");
            g.emit("unreachable");
        }
        (None, AbortMode::Label(rollback)) => g.emit(&format!("br label %{rollback}")),
        (None, AbortMode::Flag) => {
            let slot = abort_slot(g)?;
            g.emit(&format!("store i32 1, ptr {slot}"));
            g.emit("ret void");
        }
    }
    Ok(())
}
