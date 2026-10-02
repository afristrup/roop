use crate::{Ctx, Dir, Env, LeanError, Out, lean_block, lean_expr};
use roop_syntax::{Block, Expr};

/// Backward, the exit assertion selects the branch and the entry condition
/// becomes the assertion, as in the compiled code.
pub fn lean_if(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    cond: &Expr,
    then_block: &Block,
    else_block: &Block,
    exit: &Expr,
    dir: Dir,
) -> Result<(), LeanError> {
    let (entry, assertion) = match dir {
        Dir::Forward => (cond, exit),
        Dir::Backward => (exit, cond),
    };
    out.line(&format!("if {} then", lean_expr(cx, env, entry)?));
    out.indent += 1;
    lean_block(cx, env, out, then_block, dir)?;
    out.indent -= 1;
    out.line("else");
    out.indent += 1;
    lean_block(cx, env, out, else_block, dir)?;
    out.indent -= 1;
    out.line(&format!(
        "Roop.check {} Roop.Fail.assertion",
        lean_expr(cx, env, assertion)?
    ));
    Ok(())
}
