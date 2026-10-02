use roop_syntax::Expr;

pub fn expr_variants<'a>(expr: &'a Expr, out: &mut Vec<(&'a str, &'a str)>) {
    match expr {
        Expr::Variant(e, v) => out.push((e, v)),
        Expr::Unary(_, inner) => expr_variants(inner, out),
        Expr::Binary(lhs, _, rhs) => {
            expr_variants(lhs, out);
            expr_variants(rhs, out);
        }
        Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Place(_) => {}
    }
}
