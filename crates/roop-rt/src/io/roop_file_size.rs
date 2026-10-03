use crate::io::{errno, path_of};

/// The size of a file in bytes in `out`, or a negative error.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_file_size(path: *const u8, len: *const i64, out: *mut i64) {
    let path = unsafe { path_of(path, *len) };
    unsafe { *out = std::fs::metadata(path).map_or_else(|e| errno(&e), |m| m.len() as i64) };
}
