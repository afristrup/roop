use crate::io::path_of;
use crate::world::{Entry, errno_of, file_bytes, journal, require_zero};

/// Removes a file, keeping what it held for `roop_fs_remove_inv`. `status` is 0
/// or a negative error.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_remove(path: *const u8, len: *const i64, status: *mut i64) {
    unsafe { require_zero(status as *const u8, 8, "the status") };
    let path = unsafe { path_of(path, *len) };
    let old = file_bytes(&path);
    match std::fs::remove_file(&path) {
        Ok(()) => journal(Entry::Removed {
            path,
            old: old.unwrap_or_default(),
        }),
        Err(e) => {
            journal(Entry::Failed);
            unsafe { *status = errno_of(&e) };
        }
    }
}
