use crate::io::path_of;
use crate::world::{Entry, errno_of, journal, require_zero};

/// Removes an empty directory. `status` is 0 or a negative error.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_remove_dir(path: *const u8, len: *const i64, status: *mut i64) {
    unsafe { require_zero(status as *const u8, 8, "the status") };
    let path = unsafe { path_of(path, *len) };
    match std::fs::remove_dir(&path) {
        Ok(()) => journal(Entry::RemovedDir { path }),
        Err(e) => {
            journal(Entry::Failed);
            unsafe { *status = errno_of(&e) };
        }
    }
}
