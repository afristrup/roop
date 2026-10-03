use roop_syntax::{BinOp, Expr};

/// The value of an integer expression made of literals.
pub fn const_int(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Int(n) => Some(*n),
        Expr::Binary(lhs, op, rhs) => {
            let (lhs, rhs) = (const_int(lhs)?, const_int(rhs)?);
            match op {
                BinOp::Add => lhs.checked_add(rhs),
                BinOp::Sub => lhs.checked_sub(rhs),
                BinOp::Mul => lhs.checked_mul(rhs),
                _ => None,
            }
        }
        _ => None,
    }
}
