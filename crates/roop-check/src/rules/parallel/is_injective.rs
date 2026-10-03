use roop_syntax::{BinOp, Expr, Place};

/// Distinct values of `var` give distinct values of `expr`.
pub fn is_injective(expr: &Expr, var: &str) -> bool {
    match expr {
        Expr::Place(Place::Var(name)) => name == var,
        Expr::Binary(lhs, BinOp::Add | BinOp::Sub, rhs) => {
            (is_injective(lhs, var) && matches!(**rhs, Expr::Int(_)))
                || (is_injective(rhs, var) && matches!(**lhs, Expr::Int(_)))
        }
        Expr::Binary(lhs, BinOp::Mul, rhs) => {
            (is_injective(lhs, var) && matches!(**rhs, Expr::Int(c) if c != 0))
                || (is_injective(rhs, var) && matches!(**lhs, Expr::Int(c) if c != 0))
        }
        _ => false,
    }
}
