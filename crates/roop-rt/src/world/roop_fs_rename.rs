use crate::io::path_of;
use crate::world::{Entry, errno_of, file_bytes, journal, require_zero};

/// Renames a file or directory. A file it replaces is kept for the inverse.
/// `status` is 0 or a negative error.
///
/// # Safety
/// The pointers must be valid, each path for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_rename(
    source: *const u8,
    source_len: *const i64,
    target: *const u8,
    target_len: *const i64,
    status: *mut i64,
) {
    unsafe { require_zero(status as *const u8, 8, "the status") };
    let (from, to) = unsafe { (path_of(source, *source_len), path_of(target, *target_len)) };
    let overwritten = file_bytes(&to);
    match std::fs::rename(&from, &to) {
        Ok(()) => journal(Entry::Renamed {
            from,
            to,
            overwritten,
        }),
        Err(e) => {
            journal(Entry::Failed);
            unsafe { *status = errno_of(&e) };
        }
    }
}
