use crate::world::zero_bytes;

/// # Safety
/// The pointers must be valid, `buf` for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_read_inv(
    _path: *const u8,
    _len: *const i64,
    buf: *mut u8,
    cap: *const i64,
    got: *mut i64,
    status: *mut i64,
) {
    unsafe {
        zero_bytes(buf, (*cap).max(0) as usize);
        *got = 0;
        *status = 0;
    }
}
