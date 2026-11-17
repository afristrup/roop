/// How far the exact interpreter unrolls an unbounded loop: it stops when the
/// mass still in the loop is below `epsilon` or after `iterations` rounds,
/// and fails when the program has more than `stores` classical states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub iterations: usize,
    pub epsilon: f64,
    pub stores: usize,
    pub qubits: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            iterations: 10_000,
            epsilon: 1e-12,
            stores: 100_000,
            qubits: 10,
        }
    }
}
