use crate::{Expr, Visitor, walk_place};

pub fn walk_expr(v: &mut dyn Visitor, expr: &mut Expr) {
    v.expr(expr);
    match expr {
        Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Empty => {}
        Expr::Variant(name, _) => v.name(name),
        Expr::Place(place) => walk_place(v, place),
        Expr::Unary(_, inner) => walk_expr(v, inner),
        Expr::Binary(left, _, right) => {
            walk_expr(v, left);
            walk_expr(v, right);
        }
    }
}
