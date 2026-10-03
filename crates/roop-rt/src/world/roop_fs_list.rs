use crate::io::path_of;
use crate::world::{errno_of, require_zero};
use std::os::unix::ffi::OsStrExt;

/// The names in a directory, in order, each followed by a newline, as many as
/// fit in `cap` bytes. `got` is the number of bytes and `status` is 0 or a
/// negative error. The results start zero.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes and `buf` for `cap`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_fs_list(
    path: *const u8,
    len: *const i64,
    buf: *mut u8,
    cap: *const i64,
    got: *mut i64,
    status: *mut i64,
) {
    let cap = unsafe { *cap }.max(0) as usize;
    unsafe {
        require_zero(buf, cap, "the buffer");
        require_zero(got as *const u8, 8, "the length");
        require_zero(status as *const u8, 8, "the status");
    }
    let path = unsafe { path_of(path, *len) };
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            unsafe { *status = errno_of(&e) };
            return;
        }
    };
    let mut names: Vec<Vec<u8>> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().as_bytes().to_vec())
        .collect();
    names.sort();
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
