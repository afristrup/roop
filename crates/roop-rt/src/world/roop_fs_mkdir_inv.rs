use crate::world::{Entry, last_entry, refuse};

/// Removes the directories the call made, the deepest first.
///
/// # Safety
/// `status` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_mkdir_inv(_path: *const u8, _len: *const i64, status: *mut i64) {
    match last_entry() {
        Entry::Failed => {}
        Entry::MadeDirs { created } => {
            for dir in created.iter().rev() {
                if let Err(e) = std::fs::remove_dir(dir) {
                    refuse(&format!("cannot remove {}: {e}", dir.display()));
                }
            }
        }
        _ => refuse("the last file operation was not a mkdir"),
    }
    unsafe { *status = 0 };
}
