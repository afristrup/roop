use crate::{AbortMode, CodegenError, FnGen, Value, abort_slot, bool_type, ptr_type, same_type};

/// Where a failed assertion goes. Hosts trap, unless a `try` is catching, in
/// which case it rolls back. Device kernels cannot trap; they record the
/// failure in the error buffer and stop, and the host runtime reports it.
pub fn gen_assert(g: &mut FnGen, cond: &Value) -> Result<(), CodegenError> {
    same_type(&bool_type(), &cond.ty)?;
    let (ok, fail) = (g.fresh("L"), g.fresh("L"));
    g.emit(&format!("br i1 {}, label %{ok}, label %{fail}", cond.reg));
    g.label(&fail);
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
    g.label(&ok);
    Ok(())
}
