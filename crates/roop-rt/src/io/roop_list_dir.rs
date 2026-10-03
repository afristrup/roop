use crate::io::{errno, path_of};
use std::os::unix::ffi::OsStrExt;

/// Writes the names in a directory into `buf`, each followed by a newline, in
/// order, as many as fit in `cap` bytes. `got` gets the byte count, or a
/// negative error.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes and `buf` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_list_dir(
    path: *const u8,
    len: *const i64,
    buf: *mut u8,
    cap: *const i64,
    got: *mut i64,
) {
    let path = unsafe { path_of(path, *len) };
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            unsafe { *got = errno(&e) };
            return;
        }
    };
    let mut names: Vec<Vec<u8>> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().as_bytes().to_vec())
        .collect();
    names.sort();
    let cap = unsafe { *cap }.max(0) as usize;
    let mut used = 0;
    for name in names {
        if used + name.len() + 1 > cap {
            break;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(name.as_ptr(), buf.add(used), name.len());
            *buf.add(used + name.len()) = b'\n';
        }
        used += name.len() + 1;
    }
    unsafe { *got = used as i64 };
}
