use crate::quantum::{Amp, MAX_QUBITS, device};
use crate::world::refuse;

/// A fresh qubit in |0⟩, its handle written to the zero place `q`.
///
/// # Safety
/// `q` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_q_alloc(q: *mut i64) {
    if unsafe { *q } != 0 {
        refuse("a qubit ancilla must start at zero");
    }
    let mut d = device();
    if d.handles.len() == MAX_QUBITS {
        refuse(&format!("more than {MAX_QUBITS} live qubits"));
    }
    let n = d.amps.len();
    d.amps.resize(2 * n, Amp::default());
    let handle = d.next;
    d.next += 1;
    d.handles.push(handle);
    unsafe { *q = handle };
}
