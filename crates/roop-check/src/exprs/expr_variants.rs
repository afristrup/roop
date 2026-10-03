use roop_syntax::Expr;

pub fn expr_variants<'a>(expr: &'a Expr, out: &mut Vec<(&'a str, &'a str)>) {
    match expr {
        Expr::Variant(e, v) => out.push((e, v)),
        Expr::Unary(_, inner) | Expr::Cast(inner, _) => expr_variants(inner, out),
        Expr::Binary(lhs, _, rhs) => {
            expr_variants(lhs, out);
            expr_variants(rhs, out);
        }
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::Byte(_)
        | Expr::Str(_)
        | Expr::Empty
        | Expr::Place(_) => {}
    }
}
