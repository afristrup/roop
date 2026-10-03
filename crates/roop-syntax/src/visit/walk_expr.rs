use crate::{Expr, Visitor, walk_place, walk_type};

pub fn walk_expr(v: &mut dyn Visitor, expr: &mut Expr) {
    v.expr(expr);
    match expr {
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::Byte(_)
        | Expr::Str(_)
        | Expr::Empty => {}
        Expr::Variant(name, _) => v.name(name),
        Expr::Place(place) => walk_place(v, place),
        Expr::Unary(_, inner) => walk_expr(v, inner),
        Expr::Cast(inner, ty) => {
            walk_expr(v, inner);
            walk_type(v, ty);
        }
        Expr::Binary(left, _, right) => {
            walk_expr(v, left);
            walk_expr(v, right);
        }
    }
}
