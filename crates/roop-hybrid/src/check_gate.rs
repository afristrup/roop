use crate::{GateName, HybridError, Kind, Signature, lookup};

/// A gate needs its number of qubits, each declared, and no qubit twice.
pub fn check_gate(
    gate: GateName,
    qubits: &[String],
    sig: &Signature,
    line: usize,
) -> Result<(), HybridError> {
    if qubits.len() != gate.arity() {
        return Err(HybridError::Arity {
            line,
            gate: format!("{gate:?}"),
            expected: gate.arity(),
            found: qubits.len(),
        });
    }
    if let GateName::R(k) = gate
        && k > 62
    {
        return Err(HybridError::Rotation { line, k: k.into() });
    }
    for (i, q) in qubits.iter().enumerate() {
        lookup(sig, q, Kind::Qubit, line)?;
        if qubits[..i].contains(q) {
            return Err(HybridError::SharedQubit {
                line,
                qubit: q.clone(),
            });
        }
    }
    Ok(())
}
