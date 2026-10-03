use crate::io::path_of;
use crate::world::{Entry, errno_of, journal, require_zero};
use std::io::Write;

/// Adds the first `blen` bytes of `buf` to the end of a file, making it if it
/// is not there. `status` is 0 or a negative error.
///
/// # Safety
/// The pointers must be valid, each for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_append(
    path: *const u8,
    len: *const i64,
    buf: *const u8,
    blen: *const i64,
    status: *mut i64,
) {
    unsafe { require_zero(status as *const u8, 8, "the status") };
    let path = unsafe { path_of(path, *len) };
    let added = unsafe { std::slice::from_raw_parts(buf, (*blen).max(0) as usize) }.to_vec();
    let old_len = std::fs::metadata(&path).ok().map(|m| m.len());
    let result = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .and_then(|mut f| f.write_all(&added));
    match result {
        Ok(()) => journal(Entry::Appended {
            path,
            old_len,
            added,
        }),
        Err(e) => {
            journal(Entry::Failed);
            unsafe { *status = errno_of(&e) };
        }
    }
}
