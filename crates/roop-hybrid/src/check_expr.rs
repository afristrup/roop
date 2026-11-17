use crate::{Expr, HybridError, Kind, Signature, lookup};

/// Fails unless `expr` is an expression of kind `expected`. The literals 0 and
/// 1 are bits as well as ints, so `c := 1` sets a bit.
pub fn check_expr(
    expr: &Expr,
    expected: Kind,
    sig: &Signature,
    line: usize,
) -> Result<(), HybridError> {
    let wrong = || Err(HybridError::Type { line, expected });
    let both = |a: &Expr, b: &Expr, kind| {
        check_expr(a, kind, sig, line)?;
        check_expr(b, kind, sig, line)
    };
    match (expr, expected) {
        (Expr::Num(_), Kind::Int) | (Expr::Num(0 | 1), Kind::Bit) | (Expr::Bool(_), Kind::Bit) => {
            Ok(())
        }
        (Expr::Var(name), _) => lookup(sig, name, expected, line),
        (Expr::Add(a, b) | Expr::Mul(a, b) | Expr::Pow(a, b), Kind::Int) => both(a, b, Kind::Int),
        (Expr::Le(a, b) | Expr::Eq(a, b), Kind::Bit) => both(a, b, Kind::Int),
        (Expr::And(a, b) | Expr::Or(a, b), Kind::Bit) => both(a, b, Kind::Bit),
        (Expr::Not(a), Kind::Bit) => check_expr(a, Kind::Bit, sig, line),
        _ => wrong(),
    }
}
