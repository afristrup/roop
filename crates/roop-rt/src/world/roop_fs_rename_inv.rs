use crate::world::{Entry, last_entry, refuse, restore_bytes};

/// Moves the file or directory back, and puts back a file it had replaced.
///
/// # Safety
/// `status` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_rename_inv(
    _source: *const u8,
    _source_len: *const i64,
    _target: *const u8,
    _target_len: *const i64,
    status: *mut i64,
) {
    match last_entry() {
        Entry::Failed => {}
        Entry::Renamed {
            from,
            to,
            overwritten,
        } => {
            if from.exists() || !to.exists() {
                refuse("the renamed files were changed since the program renamed them");
            }
            if let Err(e) = std::fs::rename(&to, &from) {
                refuse(&format!("cannot restore {}: {e}", from.display()));
            }
            if overwritten.is_some() {
                restore_bytes(&to, &overwritten);
            }
        }
        _ => refuse("the last file operation was not a rename"),
    }
    unsafe { *status = 0 };
}
