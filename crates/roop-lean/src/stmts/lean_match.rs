use crate::{Ctx, Dir, Env, LeanError, Out, lean_block, lean_expr, pattern_test};
use roop_syntax::{Expr, MatchArm};

/// Forward, patterns select the arm and its exit assertion is checked.
/// Backward, the exit assertions select the arm and the pattern is checked.
pub fn lean_match(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    scrutinee: &Expr,
    arms: &[MatchArm],
    dir: Dir,
) -> Result<(), LeanError> {
    let held = out.fresh("__scrut");
    out.line(&format!("let {held} := {}", lean_expr(cx, env, scrutinee)?));
    for (i, arm) in arms.iter().enumerate() {
        let select = match dir {
            Dir::Forward => pattern_test(&held, &arm.pattern)?,
            Dir::Backward => lean_expr(cx, env, &arm.exit)?,
        };
        out.line(&format!(
            "{}if {select} then",
            if i == 0 { "" } else { "else " }
        ));
        out.indent += 1;
        lean_block(cx, env, out, &arm.body, dir)?;
        let assertion = match dir {
            Dir::Forward => lean_expr(cx, env, &arm.exit)?,
            Dir::Backward => pattern_test(&lean_expr(cx, env, scrutinee)?, &arm.pattern)?,
        };
        out.line(&format!("Roop.check {assertion} Roop.Fail.assertion"));
        out.indent -= 1;
    }
    out.line("else");
    out.indent += 1;
    out.line("throw Roop.Fail.assertion");
    out.indent -= 1;
    Ok(())
}
