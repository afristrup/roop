use crate::{Ctx, Env, LeanError, lean_expr, lean_type};
use roop_syntax::{Expr, Type};

/// The start value of an ancilla. `empty` needs its type spelled out.
pub fn init_text(cx: &Ctx, env: &Env, ty: &Type, init: &Expr) -> Result<String, LeanError> {
    if *init == Expr::Empty {
        if !matches!(ty, Type::Stack(..)) {
            return Err(LeanError::Unsupported("`empty` needs a stack type".into()));
        }
        return Ok(format!("(Roop.Stack.empty : {})", lean_type(ty)));
    }
    lean_expr(cx, env, init)
}
