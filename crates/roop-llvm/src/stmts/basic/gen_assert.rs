use crate::{CodegenError, FnGen, Value, bool_type, ptr_type, same_type};

/// Hosts trap on a failed assertion. Device kernels cannot, so they record
/// the failure in the error buffer and stop; the host runtime reports it.
pub fn gen_assert(g: &mut FnGen, cond: &Value) -> Result<(), CodegenError> {
    same_type(&bool_type(), &cond.ty)?;
    let (ok, fail) = (g.fresh("L"), g.fresh("L"));
    g.emit(&format!("br i1 {}, label %{ok}, label %{fail}", cond.reg));
    g.label(&fail);
    match g.error_flag.clone() {
        Some(flag) => {
            let ptr = ptr_type(g.dialect, "i32", 1);
            g.emit(&format!("store i32 1, {ptr} {flag}"));
            g.emit("ret void");
        }
        None => {
            g.emit("call void @llvm.trap()");
            g.emit("unreachable");
        }
    }
    g.label(&ok);
    Ok(())
}
