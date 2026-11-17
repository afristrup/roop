use crate::quantum::{device, position};
use crate::world::refuse;

/// Ends a qubit ancilla. It must be back in |0⟩, which also means it is not
/// entangled with anything: otherwise the ancilla was not restored and
/// dropping it would lose information. A place emptied by `keep` holds no
/// qubit, which stays with the world.
///
/// # Safety
/// `q` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_q_free(q: *mut i64) {
    let handle = unsafe { *q };
    if handle == 0 {
        return;
    }
    let mut d = device();
    let p = position(&d, handle);
    let bit = 1usize << p;
    let one: f64 = (0..d.amps.len())
        .filter(|i| i & bit != 0)
        .map(|i| d.amps[i].norm())
        .sum();
    if one > 1e-9 {
        refuse(&format!(
            "a qubit ancilla ended with probability {one:.3e} of being 1"
        ));
    }
    let low = bit - 1;
    let amps: Vec<_> = (0..d.amps.len() / 2)
        .map(|i| d.amps[(i & low) | ((i & !low) << 1)])
        .collect();
    d.amps = amps;
    d.handles.remove(p);
    unsafe { *q = 0 };
}
