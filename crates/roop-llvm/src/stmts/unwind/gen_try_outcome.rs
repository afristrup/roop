use crate::{CodegenError, Dialect, Dir, FnGen, emit_failure, gen_tagged_try};
use roop_syntax::{Block, Place};

/// A `try` with an outcome met outside failure-atomic code. If the handler
/// itself fails, that failure is the ambient one, usually a trap.
pub fn gen_try_outcome(
    g: &mut FnGen,
    body: &Block,
    handler: &Block,
    outcome: &Place,
    dir: Dir,
) -> Result<(), CodegenError> {
    if g.dialect != Dialect::Host {
        return Err(CodegenError::Unsupported("try inside a GPU kernel"));
    }
    let (fail, after) = (g.fresh("L"), g.fresh("L"));
    gen_tagged_try(g, body, handler, outcome, dir, &fail)?;
    g.emit(&format!("br label %{after}"));
    g.label(&fail);
    emit_failure(g)?;
    g.label(&after);
    Ok(())
}
