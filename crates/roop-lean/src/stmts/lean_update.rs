use crate::{
    Ctx, Dir, Env, LeanError, Out, assign_place, is_bool, is_float, lean_expr, place_type,
    read_place,
};
use roop_syntax::{Expr, Place, UpdateOp};

/// `x += e` becomes `x := x + e`; backward it is the inverse operation.
pub fn lean_update(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    target: &Place,
    op: UpdateOp,
    value: &Expr,
    dir: Dir,
) -> Result<(), LeanError> {
    let op = if dir == Dir::Backward { op.inverse() } else { op };
    let ty = place_type(cx, env, target)?;
    let (old, e) = (read_place(cx, env, target)?, lean_expr(cx, env, value)?);
    let new = match op {
        UpdateOp::Add => format!("({old} + {e})"),
        UpdateOp::Sub => format!("({old} - {e})"),
        UpdateOp::Xor if is_bool(&ty) => format!("(xor {old} {e})"),
        UpdateOp::Xor if !is_float(&ty) => format!("({old} ^^^ {e})"),
        UpdateOp::Xor => return Err(LeanError::Unsupported("xor of floats".into())),
    };
    assign_place(cx, env, out, target, &new)
}
