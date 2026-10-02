use crate::rename_place;
use roop_syntax::Expr;

pub fn rename_expr(expr: &Expr, from: &str, to: &str) -> Expr {
    match expr {
        Expr::Place(place) => Expr::Place(rename_place(place, from, to)),
        Expr::Unary(op, inner) => Expr::Unary(*op, Box::new(rename_expr(inner, from, to))),
        Expr::Binary(l, op, r) => Expr::Binary(
            Box::new(rename_expr(l, from, to)),
            *op,
            Box::new(rename_expr(r, from, to)),
        ),
        Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Variant(..) => expr.clone(),
    }
}
