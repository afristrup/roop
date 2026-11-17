use crate::{Gate, axes};

pub fn touches(gate: &Gate, qubits: &[usize]) -> bool {
    axes(gate).iter().any(|(q, _)| qubits.contains(q))
}
