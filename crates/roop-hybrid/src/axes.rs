use crate::{Axis, Gate};

/// The qubits of a gate, each with its axis. CNOT is Z on its control and X on
/// its target.
pub fn axes(gate: &Gate) -> Vec<(usize, Axis)> {
    match *gate {
        Gate::X(q) => vec![(q, Axis::X)],
        Gate::H(q) => vec![(q, Axis::Other)],
        Gate::Phase(q, _) => vec![(q, Axis::Z)],
        Gate::Cnot(c, t) => vec![(c, Axis::Z), (t, Axis::X)],
    }
}
