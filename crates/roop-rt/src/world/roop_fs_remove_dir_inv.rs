use crate::world::{Entry, last_entry, refuse};

/// Makes the directory again.
///
/// # Safety
/// `status` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_remove_dir_inv(
    _path: *const u8,
    _len: *const i64,
    status: *mut i64,
) {
    match last_entry() {
        Entry::Failed => {}
        Entry::RemovedDir { path } => {
            if let Err(e) = std::fs::create_dir(&path) {
                refuse(&format!("cannot restore {}: {e}", path.display()));
            }
        }
        _ => refuse("the last file operation was not a directory removal"),
    }
    unsafe { *status = 0 };
}
