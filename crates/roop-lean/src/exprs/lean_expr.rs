use crate::{Ctx, Env, LeanError, esc, esc_ty, expr_type, is_float, read_place};
use roop_syntax::{BinOp, Expr, UnOp};

/// A Lean expression for a roop expression. Reads of array elements use
/// `(\u{2190} ...)`, so the result must sit inside a `do` block.
pub fn lean_expr(cx: &Ctx, env: &Env, expr: &Expr) -> Result<String, LeanError> {
    Ok(match expr {
        Expr::Int(i) if *i < 0 => format!("(({i} : Roop.I64))"),
        Expr::Int(i) => format!("({i} : Roop.I64)"),
        Expr::Float(f) => format!("({f:?} : Float)"),
        Expr::Bool(b) => b.to_string(),
        Expr::Variant(e, v) => format!("{}.{}", esc_ty(e), esc(v)),
        Expr::Place(place) => read_place(cx, env, place)?,
        Expr::Unary(UnOp::Neg, inner) => format!("(-{})", lean_expr(cx, env, inner)?),
        Expr::Unary(UnOp::Not, inner) => format!("(!{})", lean_expr(cx, env, inner)?),
        Expr::Binary(lhs, op, rhs) => {
            let float = is_float(&expr_type(cx, env, lhs)?);
            let (l, r) = (lean_expr(cx, env, lhs)?, lean_expr(cx, env, rhs)?);
            match (op, float) {
                (BinOp::Add, _) => format!("({l} + {r})"),
                (BinOp::Sub, _) => format!("({l} - {r})"),
                (BinOp::Mul, _) => format!("({l} * {r})"),
                (BinOp::Div, true) => format!("({l} / {r})"),
                (BinOp::Div, false) => format!("(BitVec.sdiv {l} {r})"),
                (BinOp::Rem, false) => format!("(BitVec.srem {l} {r})"),
                (BinOp::Rem, true) => {
                    return Err(LeanError::Unsupported("remainder of floats".into()));
                }
                (BinOp::Eq, _) => format!("({l} == {r})"),
                (BinOp::Ne, _) => format!("({l} != {r})"),
                (BinOp::Lt, true) => format!("(decide ({l} < {r}))"),
                (BinOp::Le, true) => format!("(decide ({l} \u{2264} {r}))"),
                (BinOp::Gt, true) => format!("(decide ({l} > {r}))"),
                (BinOp::Ge, true) => format!("(decide ({l} \u{2265} {r}))"),
                (BinOp::Lt, false) => format!("(BitVec.slt {l} {r})"),
                (BinOp::Le, false) => format!("(BitVec.sle {l} {r})"),
                (BinOp::Gt, false) => format!("(BitVec.slt {r} {l})"),
                (BinOp::Ge, false) => format!("(BitVec.sle {r} {l})"),
                (BinOp::And, _) => format!("({l} && {r})"),
                (BinOp::Or, _) => format!("({l} || {r})"),
            }
        }
    })
}
