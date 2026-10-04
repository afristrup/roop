/// The cells of the fixed-point chain, one more than its steps, that bring its
/// error, which shrinks by `bound` each step, below one unit of Q12 for a
/// residual of size 1.
pub fn residual_iterations(bound: f64) -> usize {
    let steps = (12.0 * std::f64::consts::LN_2 / -bound.ln()).ceil();
    (steps as usize).max(3) + 1
}
