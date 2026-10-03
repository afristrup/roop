use roop_syntax::Expr;

/// The index of an array access that is a non-negative literal.
pub fn literal_index(index: &Expr) -> Option<u64> {
    match index {
        Expr::Int(i) => u64::try_from(*i).ok(),
        _ => None,
    }
}
