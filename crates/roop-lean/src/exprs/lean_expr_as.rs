use crate::{Ctx, Env, LeanError, is_byte, lean_expr};
use roop_syntax::{Expr, Type};

/// Like `lean_expr`, except that an integer literal takes the type it is used
/// with when that is `u8`.
pub fn lean_expr_as(
    cx: &Ctx,
    env: &Env,
    expr: &Expr,
    expected: &Type,
) -> Result<String, LeanError> {
    match expr {
        Expr::Int(n) if is_byte(expected) => {
            if !(0..=255).contains(n) {
                return Err(LeanError::Unsupported(
                    "a u8 literal outside 0 to 255".into(),
                ));
            }
            Ok(format!("({n} : Roop.U8)"))
        }
        _ => lean_expr(cx, env, expr),
    }
}
