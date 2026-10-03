use crate::io::path_of;
use crate::world::require_zero;
use std::os::unix::ffi::OsStrExt;

/// Copies the environment variable named by `name` into `buf`, at most `cap`
/// bytes. `len` gets its full length, or -1 when it is not set. The results
/// start zero.
///
/// # Safety
/// The pointers must be valid, `name` for `name_len` bytes and `buf` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_getenv(
    name: *const u8,
    name_len: *const i64,
    buf: *mut u8,
    cap: *const i64,
    len: *mut i64,
) {
    let cap = unsafe { *cap }.max(0) as usize;
    unsafe {
        require_zero(buf, cap, "the buffer");
        require_zero(len as *const u8, 8, "the length");
    }
    let name = unsafe { path_of(name, *name_len) };
    let Some(value) = std::env::var_os(name) else {
        unsafe { *len = -1 };
        return;
    };
    let bytes = value.as_bytes();
    let n = bytes.len().min(cap);
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n) };
    unsafe { *len = bytes.len() as i64 };
}
