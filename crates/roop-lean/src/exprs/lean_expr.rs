use crate::{
    Ctx, Env, LeanError, esc, esc_ty, expr_type, is_byte, is_float, lean_cast, lean_expr_as,
    operand_type, read_place,
};
use roop_syntax::{BinOp, Expr, UnOp};

/// A Lean expression for a roop expression. Reads of array elements use
/// `(\u{2190} ...)`, so the result must sit inside a `do` block.
pub fn lean_expr(cx: &Ctx, env: &Env, expr: &Expr) -> Result<String, LeanError> {
    Ok(match expr {
        Expr::Int(i) if *i < 0 => format!("(({i} : Roop.I64))"),
        Expr::Int(i) => format!("({i} : Roop.I64)"),
        Expr::Float(f) => format!("({f:?} : Float)"),
        Expr::Bool(b) => b.to_string(),
        Expr::Byte(b) => format!("({b} : Roop.U8)"),
        Expr::Str(bytes) => {
            let items: Vec<String> = bytes.iter().map(|b| format!("({b} : Roop.U8)")).collect();
            format!(
                "(#v[{}] : Vector Roop.U8 {})",
                items.join(", "),
                bytes.len()
            )
        }
        Expr::Cast(inner, to) => {
            let from = expr_type(cx, env, inner)?;
            lean_cast(&lean_expr(cx, env, inner)?, &from, to)?
        }
        Expr::Empty => return Err(LeanError::Unsupported("`empty` outside an ancilla".into())),
        Expr::Variant(e, v) => format!("{}.{}", esc_ty(e), esc(v)),
        Expr::Place(place) => read_place(cx, env, place)?,
        Expr::Unary(UnOp::Neg, inner) => format!("(-{})", lean_expr(cx, env, inner)?),
        Expr::Unary(UnOp::Not, inner) => format!("(!{})", lean_expr(cx, env, inner)?),
        Expr::Binary(lhs, op, rhs) => {
            let ty = operand_type(cx, env, lhs, rhs)?;
            let (float, byte) = (is_float(&ty), is_byte(&ty));
            let (l, r) = (
                lean_expr_as(cx, env, lhs, &ty)?,
                lean_expr_as(cx, env, rhs, &ty)?,
            );
            match (op, float) {
                (BinOp::Div, false) if byte => format!("(BitVec.udiv {l} {r})"),
                (BinOp::Rem, false) if byte => format!("(BitVec.urem {l} {r})"),
                (BinOp::Lt, false) if byte => format!("(BitVec.ult {l} {r})"),
                (BinOp::Le, false) if byte => format!("(BitVec.ule {l} {r})"),
                (BinOp::Gt, false) if byte => format!("(BitVec.ult {r} {l})"),
                (BinOp::Ge, false) if byte => format!("(BitVec.ule {r} {l})"),
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
