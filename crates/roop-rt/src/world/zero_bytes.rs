/// Empties a result, the way an inverse takes it back.
///
/// # Safety
/// `p` must point at `n` writable bytes.
pub unsafe fn zero_bytes(p: *mut u8, n: usize) {
    unsafe { std::ptr::write_bytes(p, 0, n) };
}
