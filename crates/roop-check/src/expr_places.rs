use super::place_index_reads;
use roop_syntax::{Expr, Place};

pub fn expr_places<'a>(expr: &'a Expr, out: &mut Vec<&'a Place>) {
    match expr {
        Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) => {}
        Expr::Place(place) => {
            out.push(place);
            place_index_reads(place, out);
        }
        Expr::Unary(_, inner) => expr_places(inner, out),
        Expr::Binary(lhs, _, rhs) => {
            expr_places(lhs, out);
            expr_places(rhs, out);
        }
    }
}
