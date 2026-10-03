use crate::world::{Entry, file_bytes, last_entry, refuse, restore_bytes};

/// Puts the file back as it was before the write.
///
/// # Safety
/// `status` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_write_inv(
    _path: *const u8,
    _len: *const i64,
    _buf: *const u8,
    _blen: *const i64,
    status: *mut i64,
) {
    match last_entry() {
        Entry::Failed => {}
        Entry::Wrote { path, old, new } => {
            if file_bytes(&path).as_deref() != Some(new.as_slice()) {
                refuse(&format!(
                    "{} was changed since the program wrote it",
                    path.display()
                ));
            }
            restore_bytes(&path, &old);
        }
        _ => refuse("the last file operation was not a write"),
    }
    unsafe { *status = 0 };
}
