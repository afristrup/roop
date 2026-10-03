use crate::{CodegenError, FnGen, Value, bool_type, emit_failure, same_type};

/// Where a failed assertion goes: see `emit_failure`.
pub fn gen_assert(g: &mut FnGen, cond: &Value) -> Result<(), CodegenError> {
    same_type(&bool_type(), &cond.ty)?;
    let (ok, fail) = (g.fresh("L"), g.fresh("L"));
    g.emit(&format!("br i1 {}, label %{ok}, label %{fail}", cond.reg));
    g.label(&fail);
    emit_failure(g)?;
    g.label(&ok);
    Ok(())
}
