use crate::{Gate, GateName, Turns};

/// R_k = RZ(2π/2^k), up to a global phase, is a phase of 1/2^k of a turn.
pub fn lower_gate(gate: GateName, qubits: &[usize]) -> Gate {
    let phase = |log| Gate::Phase(qubits[0], Turns::new(1, log));
    match gate {
        GateName::X => Gate::X(qubits[0]),
        GateName::H => Gate::H(qubits[0]),
        GateName::Cnot => Gate::Cnot(qubits[0], qubits[1]),
        GateName::Z => phase(1),
        GateName::S => phase(2),
        GateName::T => phase(3),
        GateName::R(k) => phase(k),
    }
}
