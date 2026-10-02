use crate::{Ctx, Dir, Env, LeanError, Out, lean_stmt};
use roop_syntax::Block;

/// Backward execution runs the statements in reverse order. An empty block
/// still needs a statement in a `do`.
pub fn lean_block(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    block: &Block,
    dir: Dir,
) -> Result<(), LeanError> {
    if block.stmts.is_empty() {
        out.line("pure ()");
        return Ok(());
    }
    match dir {
        Dir::Forward => {
            for stmt in &block.stmts {
                lean_stmt(cx, env, out, stmt, dir)?;
            }
        }
        Dir::Backward => {
            for stmt in block.stmts.iter().rev() {
                lean_stmt(cx, env, out, stmt, dir)?;
            }
        }
    }
    Ok(())
}
