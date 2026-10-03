use crate::io::path_of;
use crate::world::{errno_of, require_zero};

/// Reads a file into `buf`, as much as fits in `cap` bytes. `got` is the number
/// of bytes and `status` is 0 or a negative error. The results start zero.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes and `buf` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_read(
    path: *const u8,
    len: *const i64,
    buf: *mut u8,
    cap: *const i64,
    got: *mut i64,
    status: *mut i64,
) {
    let cap = unsafe { *cap }.max(0) as usize;
    unsafe {
        require_zero(buf, cap, "the buffer");
        require_zero(got as *const u8, 8, "the length");
        require_zero(status as *const u8, 8, "the status");
    }
    let path = unsafe { path_of(path, *len) };
    match std::fs::read(path) {
        Ok(bytes) => {
            let n = bytes.len().min(cap);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
                *got = n as i64;
            }
        }
        Err(e) => unsafe { *status = errno_of(&e) },
    }
}
