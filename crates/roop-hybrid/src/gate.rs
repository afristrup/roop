use crate::Turns;

/// A gate on qubit indices. Z, S, T and R_k are all `Phase`, which applies
/// e^{2πi·t} to |1⟩.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    X(usize),
    H(usize),
    Cnot(usize, usize),
    Phase(usize, Turns),
}
