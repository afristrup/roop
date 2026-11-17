use crate::{CodegenError, FnGen, Slot, qubit_addrs};

/// Runs `body` between allocating every qubit of an ancilla and freeing them
/// in reverse, which checks that each is back in |0⟩. Nothing is emitted for
/// an ancilla without qubits.
pub fn with_qubits(
    g: &mut FnGen,
    slot: &Slot,
    body: impl FnOnce(&mut FnGen) -> Result<(), CodegenError>,
) -> Result<(), CodegenError> {
    let qubits = qubit_addrs(g, &slot.ty, &slot.addr)?;
    for q in &qubits {
        g.emit(&format!("call void @roop_q_alloc(ptr {q})"));
    }
    body(g)?;
    for q in qubits.iter().rev() {
        g.emit(&format!("call void @roop_q_free(ptr {q})"));
    }
    Ok(())
}
