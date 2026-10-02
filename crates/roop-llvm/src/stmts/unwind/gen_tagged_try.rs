use crate::{
    CodegenError, Dir, FnGen, Kind, check_zero, gen_place, gen_unwinding, kind_of,
    llvm_type,
};
use roop_syntax::{Block, Place};

/// `try { body } catch_rollback { handler } -> outcome;` as an exception monad
/// whose outcome is kept as data. Forward, the body runs and, if it fails, is
/// undone by running what it did backward; then the handler runs on the
/// restored state and sets the outcome. Backward, the outcome picks the side to
/// undo: set means undo the handler, clear means undo the body.
pub fn gen_tagged_try(
    g: &mut FnGen,
    body: &Block,
    handler: &Block,
    outcome: &Place,
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    let slot = gen_place(g, outcome)?;
    let kind = kind_of(g.ctx, &slot.ty)?;
    let ty = llvm_type(g.ctx, &slot.ty)?;
    let (set, test) = match kind {
        Kind::Bool => ("true", "icmp ne"),
        Kind::Int => ("1", "icmp ne"),
        _ => return Err(CodegenError::InvalidOperand("the outcome must be bool or i64")),
    };
    let (zero, after) = (
        if kind == Kind::Bool { "false" } else { "0" },
        g.fresh("L"),
    );
    match dir {
        Dir::Forward => {
            check_zero(g, &slot)?;
            let body_failed = g.fresh("L");
            let outer = g.abort.clone();
            gen_unwinding(g, body, Dir::Forward, &body_failed)?;
            g.emit(&format!("br label %{after}"));
            g.label(&body_failed);
            g.abort = outer;
            gen_unwinding(g, handler, Dir::Forward, fail)?;
            g.emit(&format!("store {ty} {set}, ptr {}", slot.addr));
        }
        Dir::Backward => {
            let flag = format!("%{}", g.fresh("t"));
            g.emit(&format!("{flag} = load {ty}, ptr {}", slot.addr));
            let raised = format!("%{}", g.fresh("t"));
            g.emit(&format!("{raised} = {test} {ty} {flag}, {zero}"));
            let (undo_handler, undo_body) = (g.fresh("L"), g.fresh("L"));
            g.emit(&format!(
                "br i1 {raised}, label %{undo_handler}, label %{undo_body}"
            ));
            g.label(&undo_handler);
            gen_unwinding(g, handler, Dir::Backward, fail)?;
            g.emit(&format!("store {ty} {zero}, ptr {}", slot.addr));
            g.emit(&format!("br label %{after}"));
            g.label(&undo_body);
            gen_unwinding(g, body, Dir::Backward, fail)?;
        }
    }
    g.emit(&format!("br label %{after}"));
    g.label(&after);
    Ok(())
}
