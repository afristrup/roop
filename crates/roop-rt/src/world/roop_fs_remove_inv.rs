use crate::world::{Entry, last_entry, refuse};

/// Makes the removed file again, with what it held.
///
/// # Safety
/// `status` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_remove_inv(_path: *const u8, _len: *const i64, status: *mut i64) {
    match last_entry() {
        Entry::Failed => {}
        Entry::Removed { path, old } => {
            if path.exists() {
                refuse(&format!(
                    "{} was made again since the program removed it",
                    path.display()
                ));
            }
            if let Err(e) = std::fs::write(&path, old) {
                refuse(&format!("cannot restore {}: {e}", path.display()));
            }
        }
        _ => refuse("the last file operation was not a removal"),
    }
    unsafe { *status = 0 };
}
