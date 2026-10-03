use crate::world::refuse;

/// A result goes into a place that is zero, like a `pop` or a `recv`: the old
/// value would be lost, and the inverse could not bring it back.
///
/// # Safety
/// `p` must point at `n` readable bytes.
pub unsafe fn require_zero(p: *const u8, n: usize, what: &str) {
    let bytes = unsafe { std::slice::from_raw_parts(p, n) };
    if bytes.iter().any(|b| *b != 0) {
        refuse(&format!(
            "{what} must be zero when it receives a result from the world"
        ));
    }
}
