/// The Pauli whose algebra a gate stays in on one of its qubits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Z,
    Other,
}
