use roop_syntax::Expr;

pub fn starts_zero(init: &Expr) -> bool {
    matches!(
        init,
        Expr::Int(0) | Expr::Bool(false) | Expr::Byte(0) | Expr::Empty
    ) || matches!(init, Expr::Float(f) if *f == 0.0)
}
