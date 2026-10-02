use crate::{Ctx, Env, LeanError, place_type};
use roop_syntax::{BinOp, Expr, Type, UnOp};

pub fn expr_type(cx: &Ctx, env: &Env, expr: &Expr) -> Result<Type, LeanError> {
    let named = |n: &str| Type::Named(n.into());
    Ok(match expr {
        Expr::Int(_) => named("i64"),
        Expr::Float(_) => named("f64"),
        Expr::Bool(_) => named("bool"),
        Expr::Variant(enum_name, _) => named(enum_name),
        Expr::Place(place) => place_type(cx, env, place)?,
        Expr::Unary(UnOp::Not, _) => named("bool"),
        Expr::Unary(UnOp::Neg, inner) => expr_type(cx, env, inner)?,
        Expr::Binary(lhs, op, _) => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                expr_type(cx, env, lhs)?
            }
            _ => named("bool"),
        },
    })
}
