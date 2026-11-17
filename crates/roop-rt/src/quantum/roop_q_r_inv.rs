use crate::quantum::rotation;

/// # Safety
/// `q` and `k` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_q_r_inv(q: *const i64, k: *const i64) {
    unsafe { rotation(q, k, -1.0) }
}
