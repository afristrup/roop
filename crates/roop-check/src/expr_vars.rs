use super::place_vars;
use roop_syntax::Expr;
use std::collections::HashSet;

pub fn expr_vars<'a>(expr: &'a Expr, out: &mut HashSet<&'a str>) {
    match expr {
        Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Variant(..) => {}
        Expr::Place(place) => place_vars(place, out),
        Expr::Unary(_, inner) => expr_vars(inner, out),
        Expr::Binary(lhs, _, rhs) => {
            expr_vars(lhs, out);
            expr_vars(rhs, out);
        }
    }
}
