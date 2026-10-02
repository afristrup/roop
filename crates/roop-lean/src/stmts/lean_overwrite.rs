use crate::{Ctx, Dir, Env, LeanError, Out, assign_place, is_float, lean_expr, place_type, read_place};
use roop_syntax::{Expr, OverwriteOp, Place};

/// Destroys the old value, so it only exists going forward.
pub fn lean_overwrite(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    target: &Place,
    op: OverwriteOp,
    value: &Expr,
    dir: Dir,
) -> Result<(), LeanError> {
    if dir == Dir::Backward {
        return Err(LeanError::Unsupported("an overwrite run backward".into()));
    }
    let float = is_float(&place_type(cx, env, target)?);
    let (old, e) = (read_place(cx, env, target)?, lean_expr(cx, env, value)?);
    let new = match (op, float) {
        (OverwriteOp::Assign, _) => e,
        (OverwriteOp::Mul, _) => format!("({old} * {e})"),
        (OverwriteOp::Div, true) => format!("({old} / {e})"),
        (OverwriteOp::Div, false) => format!("(BitVec.sdiv {old} {e})"),
        (OverwriteOp::Rem, false) => format!("(BitVec.srem {old} {e})"),
        (OverwriteOp::Rem, true) => {
            return Err(LeanError::Unsupported("remainder of floats".into()));
        }
    };
    assign_place(cx, env, out, target, &new)
}
