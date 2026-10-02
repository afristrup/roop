use crate::{Ctx, Dir, Env, LeanError, Out, esc, lean_block, lean_expr};
use roop_syntax::{Block, Expr, Type};

/// A temporary that starts at `init` and must be back there when its block
/// ends. The compiler proves this statically; here it is a check that the
/// function's theorem shows never fails.
pub fn lean_ancilla(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    name: &str,
    ty: &Type,
    init: &Expr,
    body: &Block,
    dir: Dir,
) -> Result<(), LeanError> {
    out.line(&format!(
        "let mut {} := {}",
        esc(name),
        lean_expr(cx, env, init)?
    ));
    env.vars.push((name.into(), ty.clone()));
    let result = lean_block(cx, env, out, body, dir);
    env.vars.pop();
    result?;
    if !env.irreversible {
        out.ancillas += 1;
        out.line(&format!(
            "Roop.check ({} == {}) Roop.Fail.ancilla",
            esc(name),
            lean_expr(cx, env, init)?
        ));
    }
    Ok(())
}
