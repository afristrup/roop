use crate::io::{path_of, status_of};

/// Removes an empty directory; `status` is 0 or a negative error.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_remove_dir(path: *const u8, len: *const i64, status: *mut i64) {
    let path = unsafe { path_of(path, *len) };
    unsafe { *status = status_of(std::fs::remove_dir(path)) };
}
