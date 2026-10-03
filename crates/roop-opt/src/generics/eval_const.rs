use roop_syntax::{BinOp, Expr, Place, UnOp};
use std::collections::HashMap;

/// The value of an expression built from literals and generic parameters, and
/// whether it used a parameter. `None` when it depends on anything else.
pub fn eval_const(expr: &Expr, env: &HashMap<String, i64>) -> Option<(i64, bool)> {
    match expr {
        Expr::Int(n) => Some((*n, false)),
        Expr::Place(Place::Var(name)) => env.get(name).map(|n| (*n, true)),
        Expr::Unary(UnOp::Neg, inner) => {
            let (n, used) = eval_const(inner, env)?;
            Some((n.checked_neg()?, used))
        }
        Expr::Binary(left, op, right) => {
            let (a, ua) = eval_const(left, env)?;
            let (b, ub) = eval_const(right, env)?;
            let n = match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                BinOp::Div => a.checked_div(b),
                BinOp::Rem => a.checked_rem(b),
                _ => None,
            }?;
            Some((n, ua || ub))
        }
        _ => None,
    }
}
