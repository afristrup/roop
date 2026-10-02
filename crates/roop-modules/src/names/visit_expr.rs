use crate::{OnName, visit_place};
use roop_syntax::Expr;

pub fn visit_expr(expr: &mut Expr, on: OnName) {
    match expr {
        Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Empty => {}
        Expr::Variant(name, _) => on(name),
        Expr::Place(place) => visit_place(place, on),
        Expr::Unary(_, inner) => visit_expr(inner, on),
        Expr::Binary(left, _, right) => {
            visit_expr(left, on);
            visit_expr(right, on);
        }
    }
}
