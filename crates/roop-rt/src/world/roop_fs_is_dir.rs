use crate::io::path_of;
use crate::world::require_zero;

/// 1 when the path is a directory, otherwise 0. The result starts zero.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_is_dir(path: *const u8, len: *const i64, out: *mut i64) {
    unsafe { require_zero(out as *const u8, 8, "the result") };
    let path = unsafe { path_of(path, *len) };
    unsafe { *out = i64::from(path.is_dir()) };
}
