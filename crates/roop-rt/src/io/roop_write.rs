use crate::io::{count_of, with_fd};
use std::io::Write;

/// Writes the first `len` bytes of `buf` to descriptor `fd`; `written` gets
/// the count, or a negative error.
///
/// # Safety
/// The pointers must be valid, `buf` for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_write(
    fd: *const i64,
    buf: *const u8,
    len: *const i64,
    written: *mut i64,
) {
    let (fd, len) = unsafe { (*fd, *len) };
    let bytes = unsafe { std::slice::from_raw_parts(buf, len.max(0) as usize) };
    let result = unsafe { with_fd(fd, |f| f.write_all(bytes).map(|()| bytes.len())) };
    unsafe { *written = count_of(result) };
}
