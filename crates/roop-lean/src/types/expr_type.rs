use crate::{Ctx, Env, LeanError, place_type};
use roop_syntax::{BinOp, Expr, Type, UnOp};

pub fn expr_type(cx: &Ctx, env: &Env, expr: &Expr) -> Result<Type, LeanError> {
    let named = |n: &str| Type::Named(n.into());
    Ok(match expr {
        Expr::Int(_) => named("i64"),
        Expr::Float(_) => named("f64"),
        Expr::Bool(_) => named("bool"),
        Expr::Byte(_) => named("u8"),
        Expr::Str(bytes) => Type::Array(Box::new(named("u8")), bytes.len() as u64),
        Expr::Cast(_, ty) => ty.clone(),
        Expr::Empty => return Err(LeanError::Unsupported("`empty` outside an ancilla".into())),
        Expr::Variant(enum_name, _) => named(enum_name),
        Expr::Place(place) => place_type(cx, env, place)?,
        Expr::Unary(UnOp::Not, _) => named("bool"),
        Expr::Unary(UnOp::Neg, inner) => expr_type(cx, env, inner)?,
        Expr::Binary(lhs, op, rhs) => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                operand_type(cx, env, lhs, rhs)?
            }
            _ => named("bool"),
        },
    })
}

/// The type two operands share: an integer literal takes the other one's.
pub fn operand_type(cx: &Ctx, env: &Env, lhs: &Expr, rhs: &Expr) -> Result<Type, LeanError> {
    match (lhs, rhs) {
        (Expr::Int(_), Expr::Int(_)) => expr_type(cx, env, lhs),
        (Expr::Int(_), _) => expr_type(cx, env, rhs),
        _ => expr_type(cx, env, lhs),
    }
}
