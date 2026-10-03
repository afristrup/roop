use crate::world::zero_bytes;

/// # Safety
/// The pointers must be valid, `buf` for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_getenv_inv(
    _name: *const u8,
    _name_len: *const i64,
    buf: *mut u8,
    cap: *const i64,
    len: *mut i64,
) {
    unsafe {
        zero_bytes(buf, (*cap).max(0) as usize);
        *len = 0;
    }
}
