use crate::io::path_of;
use std::os::unix::ffi::OsStrExt;

/// Copies the environment variable named by `name` into `buf`, at most `cap`
/// bytes and zeroes the rest of the `cap`. `len` gets its full length, or -1 when it is not set.
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
    let name = unsafe { path_of(name, *name_len) };
    let Some(value) = std::env::var_os(name) else {
        unsafe { *len = -1 };
        return;
    };
    let bytes = value.as_bytes();
    let cap = unsafe { *cap }.max(0) as usize;
    unsafe { std::ptr::write_bytes(buf, 0, cap) };
    let n = bytes.len().min(cap);
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n) };
    unsafe { *len = bytes.len() as i64 };
}
