use std::collections::BTreeMap;

/// Expected counts over all runs: each event weighs the probability of
/// reaching it, so a T gate in a loop run twice on average counts 2.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Costs {
    pub gates: BTreeMap<&'static str, f64>,
    pub kernels: f64,
    pub measurements: f64,
    pub iterations: f64,
}
