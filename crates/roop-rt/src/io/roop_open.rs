use crate::io::{errno, path_of};
use std::fs::OpenOptions;
use std::os::fd::IntoRawFd;

/// Opens a file: mode 0 reads, 1 writes and truncates, 2 appends, 3 reads and
/// writes an existing file. `fd` gets the descriptor, or a negative error.
///
/// # Safety
/// The pointers must be valid, `path` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_open(
    path: *const u8,
    len: *const i64,
    mode: *const i64,
    fd: *mut i64,
) {
    let path = unsafe { path_of(path, *len) };
    let mut options = OpenOptions::new();
    match unsafe { *mode } {
        0 => options.read(true),
        1 => options.write(true).create(true).truncate(true),
        2 => options.append(true).create(true),
        _ => options.read(true).write(true),
    };
    let opened = options.open(path).map(|f| i64::from(f.into_raw_fd()));
    unsafe { *fd = opened.unwrap_or_else(|e| errno(&e)) };
}
