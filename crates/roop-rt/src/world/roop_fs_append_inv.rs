use crate::world::{Entry, file_bytes, last_entry, refuse};

/// Cuts what was added off the end of the file again, or removes the file if
/// the append made it.
///
/// # Safety
/// `status` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_append_inv(
    _path: *const u8,
    _len: *const i64,
    _buf: *const u8,
    _blen: *const i64,
    status: *mut i64,
) {
    match last_entry() {
        Entry::Failed => {}
        Entry::Appended {
            path,
            old_len,
            added,
        } => {
            let now = file_bytes(&path).unwrap_or_default();
            let keep = old_len.unwrap_or(0) as usize;
            if now.len() != keep + added.len() || now[keep..] != added[..] {
                refuse(&format!(
                    "{} was changed since the program appended to it",
                    path.display()
                ));
            }
            let result = match old_len {
                Some(n) => std::fs::OpenOptions::new()
                    .write(true)
                    .open(&path)
                    .and_then(|f| f.set_len(n)),
                None => std::fs::remove_file(&path),
            };
            if let Err(e) = result {
                refuse(&format!("cannot restore {}: {e}", path.display()));
            }
        }
        _ => refuse("the last file operation was not an append"),
    }
    unsafe { *status = 0 };
}
