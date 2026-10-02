use crate::{CodegenError, FnGen, Value, bool_type, same_type};

/// Trap at run time unless the assertion holds.
pub fn gen_assert(g: &mut FnGen, cond: &Value) -> Result<(), CodegenError> {
    same_type(&bool_type(), &cond.ty)?;
    let (ok, fail) = (g.fresh("L"), g.fresh("L"));
    g.emit(&format!("br i1 {}, label %{ok}, label %{fail}", cond.reg));
    g.label(&fail);
    g.emit("call void @llvm.trap()");
    g.emit("unreachable");
    g.label(&ok);
    Ok(())
}
