use crate::io::path_of;
use crate::world::{errno_of, require_zero};

/// The size of a file in bytes, or a negative error. The result starts zero.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_size(path: *const u8, len: *const i64, out: *mut i64) {
    unsafe { require_zero(out as *const u8, 8, "the result") };
    let path = unsafe { path_of(path, *len) };
    unsafe { *out = std::fs::metadata(&path).map_or_else(|e| errno_of(&e), |m| m.len() as i64) };
}
