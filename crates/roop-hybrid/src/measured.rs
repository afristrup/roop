use crate::Op;

/// The qubits that the measurements among `ops` read.
pub fn measured(ops: &[Op]) -> Vec<usize> {
    ops.iter()
        .filter_map(|op| match op {
            Op::Measure { qubit, .. } => Some(*qubit),
            _ => None,
        })
        .collect()
}
