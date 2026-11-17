use crate::quantum::{gate, phase};
use crate::world::refuse;
use std::f64::consts::TAU;

/// R_k = diag(1, e^{2πi/2^k}), run backward when `sign` is -1.
///
/// # Safety
/// `q` and `k` must be valid.
pub unsafe fn rotation(q: *const i64, k: *const i64, sign: f64) {
    let k = unsafe { *k };
    if !(0..=62).contains(&k) {
        refuse("R_k needs k from 0 to 62");
    }
    unsafe { gate(q, phase(sign * TAU / (1u64 << k) as f64)) }
}
