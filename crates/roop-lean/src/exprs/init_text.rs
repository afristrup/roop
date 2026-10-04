use crate::{Ctx, Env, LeanError, lean_expr_as, lean_type};
use roop_syntax::{Expr, Type};

/// The start value of an ancilla. `empty` needs its type spelled out, and `0`
/// stands for an array of zeros.
pub fn init_text(cx: &Ctx, env: &Env, ty: &Type, init: &Expr) -> Result<String, LeanError> {
    if *init == Expr::Empty {
        if !matches!(ty, Type::Stack(..)) {
            return Err(LeanError::Unsupported("`empty` needs a stack type".into()));
        }
        return Ok(format!("(Roop.Stack.empty : {})", lean_type(ty)));
    }
    if let (Expr::Int(0), Type::Array(elem, len)) = (init, ty) {
        let zero = array_zero(elem)?;
        return Ok(format!("(Roop.zeros {len} {zero})"));
    }
    lean_expr_as(cx, env, init, ty)
}

/// Zero for an array element, which may be an array itself.
fn array_zero(elem: &Type) -> Result<String, LeanError> {
    match elem {
        Type::Array(inner, len) => Ok(format!("(Vector.replicate {len} {})", array_zero(inner)?)),
        other => crate::lean_zero(other),
    }
}
