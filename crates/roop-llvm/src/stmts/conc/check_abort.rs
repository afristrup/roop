use crate::{AbortMode, CodegenError, FnGen, abort_slot};

/// After running outlined code from a `try`: if it raised the abort flag, take
/// the rollback path (or, inside another outlined body, return).
pub fn check_abort(g: &mut FnGen) -> Result<(), CodegenError> {
    let mode = g.abort.clone();
    if mode == AbortMode::Trap {
        return Ok(());
    }
    let slot = abort_slot(g)?;
    let flag = format!("%{}", g.fresh("t"));
    g.emit(&format!("{flag} = load i32, ptr {slot}"));
    let raised = format!("%{}", g.fresh("t"));
    g.emit(&format!("{raised} = icmp ne i32 {flag}, 0"));
    let (bail, go_on) = (g.fresh("L"), g.fresh("L"));
    match mode {
        AbortMode::Label(rollback) => g.emit(&format!(
            "br i1 {raised}, label %{rollback}, label %{go_on}"
        )),
        _ => {
            g.emit(&format!("br i1 {raised}, label %{bail}, label %{go_on}"));
            g.label(&bail);
            g.emit("ret void");
        }
    }
    g.label(&go_on);
    Ok(())
}
