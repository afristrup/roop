use crate::{CqState, Store};

/// Σ f(store)·tr(ρ) over the parts of a state, divided by its trace: the mean
/// of `f` over the runs that terminated.
pub fn expectation(state: &CqState, f: impl Fn(&Store) -> f64) -> f64 {
    let total: f64 = state.parts.iter().map(|(s, rho)| f(s) * rho.trace()).sum();
    total / state.trace()
}
