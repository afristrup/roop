use roop_syntax::{CountedLoop, Expr};

/// The number of iterations when both bounds are integer literals.
pub fn static_trip(counted: &CountedLoop) -> Option<i64> {
    match (counted.lo, counted.hi) {
        (Expr::Int(lo), Expr::Int(hi)) if hi >= lo => Some((hi - lo) / counted.step + 1),
        _ => None,
    }
}
