use crate::io::path_of;

/// 1 in `out` when the path exists, otherwise 0.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_exists(path: *const u8, len: *const i64, out: *mut i64) {
    let path = unsafe { path_of(path, *len) };
    unsafe { *out = i64::from(path.exists()) };
}
