use crate::quantum::{Amp, apply_one, device, position};

/// Applies `m` to the qubit whose handle is at `q`.
///
/// # Safety
/// `q` must be valid.
pub unsafe fn gate(q: *const i64, m: [[Amp; 2]; 2]) {
    let mut d = device();
    let p = position(&d, unsafe { *q });
    apply_one(&mut d, p, m);
}
