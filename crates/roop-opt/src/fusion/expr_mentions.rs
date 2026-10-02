use roop_syntax::{Expr, Place};

pub fn expr_mentions(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Place(place) => place_mentions(place, name),
        Expr::Unary(_, inner) => expr_mentions(inner, name),
        Expr::Binary(l, _, r) => expr_mentions(l, name) || expr_mentions(r, name),
        Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Variant(..) => false,
    }
}

fn place_mentions(place: &Place, name: &str) -> bool {
    match place {
        Place::Var(n) => n == name,
        Place::Field(base, _) => place_mentions(base, name),
        Place::Index(base, i) => place_mentions(base, name) || expr_mentions(i, name),
    }
}
